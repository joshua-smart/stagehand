use std::{
    io::{self},
    string::FromUtf8Error,
    sync::{Arc, Mutex},
};

use tracing_subscriber::fmt::MakeWriter;

pub struct MkLogCapture {
    logger: LogCapture,
}

impl MkLogCapture {
    pub fn new(logger: LogCapture) -> Self {
        Self { logger }
    }
}

impl<'a> MakeWriter<'a> for MkLogCapture {
    type Writer = LogCapture;

    fn make_writer(&'a self) -> Self::Writer {
        self.logger.clone()
    }
}

#[derive(Clone, Debug)]
pub struct LogCapture(Arc<Mutex<Vec<u8>>>);

impl LogCapture {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Vec::new())))
    }

    pub fn read(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.0.lock().expect("lock poisoned").clone())
    }
}

impl io::Write for LogCapture {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("lock poisoned").write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
