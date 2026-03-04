use std::{collections::BTreeMap, net::IpAddr, sync::mpsc::Sender};

use crate::{
    command::Command,
    data_structures::{CHANNELS_PER_UNIVERSE, level::Level, universe::Universe},
};

pub enum ControllerUiMessage {
    Update,
}

pub struct UiControllerMessage {
    pub command: Command,
    pub reply: Sender<Result<String, String>>,
}

pub type OutputData = BTreeMap<Universe, [Level; CHANNELS_PER_UNIVERSE]>;

pub enum ControllerOutputMessage {
    Start(IpAddr, OutputData),
    Stop,
    Update(OutputData),
}
