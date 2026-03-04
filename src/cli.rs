use std::path::PathBuf;

#[derive(clap::Parser)]
pub struct Args {
    pub showfile: Option<PathBuf>,
}
