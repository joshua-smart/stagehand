use std::{
    array,
    collections::BTreeMap,
    net::Ipv4Addr,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
};

use tracing::info;

use crate::{
    command::Command,
    data_structures::{
        CHANNELS_PER_UNIVERSE,
        address::{Address, AddressSet},
        index::Index,
        level::{Level, LevelSet},
        show::Show,
        universe::Universe,
    },
    messages::{ControllerOutputMessage, ControllerUiMessage, UiControllerMessage},
};

pub struct Controller {
    tui_rx: Receiver<UiControllerMessage>,
    tui_tx: Sender<ControllerUiMessage>,
    output_tx: Sender<ControllerOutputMessage>,
    show: Arc<Mutex<Show>>,
}

impl Controller {
    pub fn new(
        tui_rx: Receiver<UiControllerMessage>,
        tui_tx: Sender<ControllerUiMessage>,
        output_tx: Sender<ControllerOutputMessage>,
        show: Arc<Mutex<Show>>,
    ) -> Self {
        Self {
            tui_rx,
            tui_tx,
            output_tx,
            show,
        }
    }

    pub fn run(mut self) {
        let data = [(Universe::ONE, [Level::OUT; CHANNELS_PER_UNIVERSE])]
            .into_iter()
            .collect();

        self.output_tx
            .send(ControllerOutputMessage::Start(
                Ipv4Addr::LOCALHOST.into(),
                data,
            ))
            .unwrap();

        while let Ok(UiControllerMessage { command, reply }) = self.tui_rx.recv() {
            info!(command=?command, "recieved");
            if let Command::Quit = command {
                reply.send(Ok(String::new())).unwrap();
                break;
            }
            self.handle_command(command, reply);
        }
    }

    fn handle_command(&mut self, command: Command, reply: Sender<Result<String, String>>) {
        let res = match command {
            Command::Quit => unreachable!("quit is handled outside of this scope"),
            Command::SetAddress {
                address_set,
                level_set,
            } => self.set_address(address_set, level_set),
            Command::ClearAddress { address_set } => self.clear_address(address_set),
            Command::Save { path } => self.save(path),
        };
        reply.send(res).unwrap();
        self.tui_tx.send(ControllerUiMessage::Update).unwrap();

        self.output_tx
            .send(ControllerOutputMessage::Update(self.generate_levels()))
            .unwrap();
    }

    fn generate_levels(&self) -> BTreeMap<Universe, [Level; CHANNELS_PER_UNIVERSE]> {
        let (universes, set_addresses) = {
            let show = self.show.lock().unwrap();
            (show.universes.clone(), show.runtime.set_addresses.clone())
        };

        universes
            .into_iter()
            .map(|u| {
                (
                    u,
                    array::from_fn(|i| {
                        let index = Index::new(i as u16 + 1).expect("should always be valid");
                        set_addresses
                            .get(&Address { universe: u, index })
                            .copied()
                            .unwrap_or(Level::OUT)
                    }),
                )
            })
            .collect()
    }

    fn set_address(
        &mut self,
        address_set: AddressSet,
        level_set: LevelSet,
    ) -> Result<String, String> {
        let universe = address_set.universe();
        let indexes = address_set.indexes();

        let index_count = indexes.len();
        let index_levels = indexes
            .into_iter()
            .enumerate()
            .map(|(p, i)| match &level_set {
                LevelSet::Single(level) => (i, *level),
                LevelSet::Range(level_range) => {
                    let t = p as f64 / (index_count as f64 - 1.0);
                    (i, level_range.interpolate(t))
                }
            });

        let mut show = self.show.lock().unwrap();
        let universes = show.universes.clone();
        let set_addresses = &mut show.runtime.set_addresses;

        if !universes.contains(&universe) {
            return Err(format!(
                "Universe {} is not enabled, enabled universes: {}",
                u16::from(universe),
                universes
                    .into_iter()
                    .map(|u| u16::from(u).to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        };

        for (i, l) in index_levels {
            set_addresses.insert(Address { universe, index: i }, l);
        }

        Ok(String::new())
    }

    fn clear_address(&mut self, address_set: AddressSet) -> Result<String, String> {
        let universe = address_set.universe();
        let indexes = address_set.indexes();

        let mut show = self.show.lock().unwrap();
        let universes = show.universes.clone();
        let set_addresses = &mut show.runtime.set_addresses;

        if !universes.contains(&universe) {
            return Err(format!(
                "Universe {} is not enabled, enabled universes: {}",
                u16::from(universe),
                universes
                    .into_iter()
                    .map(|u| u16::from(u).to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        };

        for i in indexes {
            set_addresses.remove(&Address { universe, index: i });
        }
        Ok(String::new())
    }

    fn save(&mut self, new_path: Option<PathBuf>) -> Result<String, String> {
        let show = {
            let show = self.show.lock().unwrap();
            show.clone()
        };
        let current_path = show.runtime.showfile.clone();

        let path_to_save = match (current_path, new_path) {
            (None, None) => return Err("no path set".to_string()),
            (Some(path), None) => path,
            (_, Some(path)) => path,
        };

        let show_str = toml::to_string(&show).map_err(|e| e.to_string())?;

        std::fs::write(&path_to_save, show_str).map_err(|e| e.to_string())?;

        let mut show = self.show.lock().unwrap();
        show.runtime.showfile = Some(path_to_save);

        Ok(String::new())
    }
}
