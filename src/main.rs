use clap::Parser;
use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self},
    },
    thread::{self},
};
use tracing::trace;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt as _};

use crate::logging::{LogCapture, MkLogCapture};
use crate::{
    cli::Args, controller::Controller, data_structures::show::Show, dmx_output::DmxOutputRunner,
    tui_runner::TuiRunner,
};

mod cli;
mod command;
mod controller;
mod data_structures;
mod dmx_output;
mod logging;
mod messages;
mod tui;
mod tui_runner;

fn main() -> anyhow::Result<()> {
    // Parse cli args
    let Args { showfile } = Args::parse();

    let log_capture = LogCapture::new();

    // Setup Logging
    let log_file = std::fs::File::create("log")?;
    let file_subscriber = tracing_subscriber::fmt::layer().with_writer(log_file);
    let code_subscriber = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(MkLogCapture::new(log_capture.clone()));
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(code_subscriber)
        .init();

    trace!(?showfile, "loading show");
    let mut show: Show = match &showfile {
        Some(showfile) => toml::from_str(&std::fs::read_to_string(showfile)?)?,
        None => Show::default(),
    };

    show.runtime.showfile = showfile;
    trace!(?show, "loaded show");
    show.runtime.logs = Some(log_capture);

    let show = Arc::new(Mutex::new(show));

    let (controller_tui_tx, controller_tui_rx) = mpsc::channel();
    let (tui_controller_tx, tui_controller_rx) = mpsc::channel();
    let (controller_output_tx, controller_output_rx) = mpsc::channel();

    let controller = Controller::new(
        tui_controller_rx,
        controller_tui_tx,
        controller_output_tx,
        show.clone(),
    );

    let dmx_output = DmxOutputRunner::new(controller_output_rx);

    let tui = TuiRunner::new(controller_tui_rx, tui_controller_tx, show);

    thread::scope(|s| {
        trace!("starting stagehand thread");
        s.spawn({
            move || {
                controller.run();
                trace!("stagehand thread exiting");
            }
        });
        trace!("starting tui thread");
        s.spawn({
            move || {
                tui.run().unwrap();
                trace!("tui thread exiting");
            }
        });
        trace!("starting sacn thread");
        s.spawn(move || {
            dmx_output.run();
            trace!("sacn thread exiting");
        });
    });
    trace!("exited successfully");

    Ok(())
}
