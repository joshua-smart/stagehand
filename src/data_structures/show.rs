use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{
    data_structures::{address::Address, level::Level, universe::Universe},
    logging::LogCapture,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Show {
    pub universes: BTreeSet<Universe>,
    pub output: Output,
    #[serde(skip)]
    pub runtime: Runtime,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Output {
    address: IpAddr,
}

#[derive(Debug, Default, Clone)]
pub struct Runtime {
    pub showfile: Option<PathBuf>,
    pub set_addresses: BTreeMap<Address, Level>,
    pub logs: Option<LogCapture>,
}

impl Default for Show {
    fn default() -> Self {
        Show {
            universes: BTreeSet::from([
                Universe::ONE,
                Universe::new(2).unwrap(),
                Universe::new(3).unwrap(),
            ]),
            output: Output {
                address: Ipv4Addr::LOCALHOST.into(),
            },
            runtime: Runtime {
                showfile: None,
                set_addresses: BTreeMap::new(),
                logs: None,
            },
        }
    }
}
