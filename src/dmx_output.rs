use std::{
    collections::BTreeSet,
    net::{IpAddr, SocketAddr},
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use sacn::{error::errors::SacnError, packet::ACN_SDT_MULTICAST_PORT, source::SacnSource};
use tracing::warn;

use crate::{data_structures::CHANNELS_PER_UNIVERSE, messages::ControllerOutputMessage};

static SACN_NAME: &str = "stagehand";
static SEND_DELAY: Duration = Duration::from_millis(20);

struct DmxOutput {
    source: SacnSource,
    data: Vec<(u16, [u8; CHANNELS_PER_UNIVERSE])>,
}

impl DmxOutput {
    pub fn new(
        address: IpAddr,
        data: Vec<(u16, [u8; CHANNELS_PER_UNIVERSE])>,
    ) -> Result<Self, SacnError> {
        let source = SacnSource::with_ip(
            SACN_NAME,
            SocketAddr::new(address, ACN_SDT_MULTICAST_PORT + 1),
        )?;
        Ok(Self { source, data })
    }

    pub fn send(&mut self) -> Result<(), SacnError> {
        if self.data.is_empty() {
            warn!("sACN output is active, but data is empty");
            return Ok(());
        }

        let mut universes = BTreeSet::new();
        let mut buf = vec![];
        for (u, u_levels) in &self.data {
            universes.insert(*u);
            buf.push(0x00);
            buf.extend_from_slice(u_levels);
        }

        // ensure universes are consistent
        let current_universes: BTreeSet<_> = self.source.universes()?.into_iter().collect();
        let universes_to_terminate = current_universes.difference(&universes);
        let universes_to_register = universes.difference(&current_universes);

        for u in universes_to_terminate {
            self.source.terminate_stream(*u, 0x00)?;
        }
        self.source.register_universes(
            &universes_to_register
                .into_iter()
                .copied()
                .collect::<Vec<_>>(),
        )?;

        self.source
            .send(
                &universes.into_iter().collect::<Vec<_>>(),
                &buf,
                None,
                None,
                None,
            )
            .unwrap();

        Ok(())
    }

    pub fn update(&mut self, data: Vec<(u16, [u8; CHANNELS_PER_UNIVERSE])>) {
        self.data = data;
    }
}

pub struct DmxOutputRunner {
    dmx_output: Option<DmxOutput>,
    controller_rx: Receiver<ControllerOutputMessage>,
}

impl DmxOutputRunner {
    pub fn new(controller_rx: Receiver<ControllerOutputMessage>) -> Self {
        Self {
            dmx_output: None,
            controller_rx,
        }
    }

    pub fn run(mut self) {
        loop {
            match self.controller_rx.recv_timeout(SEND_DELAY) {
                Ok(message) => self.handle_message(message),
                Err(RecvTimeoutError::Timeout) => (),
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if let Some(dmx_output) = &mut self.dmx_output {
                dmx_output.send().unwrap();
            }
        }
    }

    fn handle_message(&mut self, message: ControllerOutputMessage) {
        match message {
            ControllerOutputMessage::Start(address, data) => {
                let data = data
                    .into_iter()
                    .map(|(u, ls)| (u16::from(u), ls.map(u8::from)))
                    .collect();
                self.dmx_output = Some(DmxOutput::new(address, data).unwrap());
            }
            ControllerOutputMessage::Stop => todo!(),
            ControllerOutputMessage::Update(data) => {
                if let Some(dmx_output) = &mut self.dmx_output {
                    let data = data
                        .into_iter()
                        .map(|(u, ls)| (u16::from(u), ls.map(u8::from)))
                        .collect();
                    dmx_output.update(data);
                }
            }
        }
    }
}
