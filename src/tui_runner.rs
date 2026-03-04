use std::{
    io,
    sync::{
        Arc, Mutex,
        mpsc::{
            self, Receiver, RecvError, Sender,
            TryRecvError::{self, Disconnected},
        },
    },
    time::Duration,
};

use ratatui::crossterm::event;
use tracing::error;

use crate::{
    command::Command,
    data_structures::show::Show,
    messages::{ControllerUiMessage, UiControllerMessage},
    tui::Tui,
};

pub struct TuiRunner {
    controller_rx: Receiver<ControllerUiMessage>,
    controller_tx: Sender<UiControllerMessage>,
    show: Arc<Mutex<Show>>,
}

impl TuiRunner {
    pub fn new(
        controller_rx: Receiver<ControllerUiMessage>,
        controller_tx: Sender<UiControllerMessage>,
        show: Arc<Mutex<Show>>,
    ) -> Self {
        Self {
            controller_rx,
            controller_tx,
            show,
        }
    }

    pub fn run(self) -> io::Result<()> {
        let mut show = self.show.lock().unwrap().clone();
        let mut tui = Tui::new();

        let mut terminal = ratatui::init();

        loop {
            match self.controller_rx.try_recv() {
                Ok(ControllerUiMessage::Update) => {
                    show = self.show.lock().unwrap().clone();
                }
                Err(TryRecvError::Empty) => (),
                Err(Disconnected) => break,
            }

            terminal
                .draw(|frame| frame.render_stateful_widget(&mut tui, frame.area(), &mut show))?;

            if event::poll(Duration::from_millis(0))? {
                let event = event::read()?;
                if let Some(command) = tui.handle_event(event, &show) {
                    let res = self.send(command);
                    tui.command_result(res);
                };
            };
        }

        ratatui::restore();
        Ok(())
    }

    fn send(&self, command: Command) -> Result<String, String> {
        let (reply_tx, reply_rx) = mpsc::channel();

        self.controller_tx
            .send(UiControllerMessage {
                command,
                reply: reply_tx,
            })
            .unwrap();

        match reply_rx.recv() {
            Ok(res) => res,
            Err(RecvError) => {
                error!("command reply dropped");
                Err(String::new())
            }
        }
    }
}
