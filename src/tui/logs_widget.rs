use ratatui::{
    prelude::{Buffer, Rect},
    widgets::{Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::data_structures::show::Show;

pub struct LogsWidget;

impl LogsWidget {
    pub fn new() -> Self {
        Self
    }
}

impl StatefulWidget for &LogsWidget {
    type State = Show;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let logs = state
            .runtime
            .logs
            .as_ref()
            .map(|l| l.read())
            .unwrap_or(Ok(String::new()))
            .unwrap_or_else(|e| e.to_string());

        Paragraph::new(logs)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
