use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::crossterm::event::{Event, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Widget};

use crate::command::Command;
use crate::command::parse_command;

pub enum CommandBufferWidget {
    Success { message: Option<String> },
    Error { message: String },
    Idle { buf: String },
}

impl Widget for &CommandBufferWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let color = match self {
            CommandBufferWidget::Success { .. } => Color::Green,
            CommandBufferWidget::Error { .. } => Color::Red,
            CommandBufferWidget::Idle { .. } => Color::Reset,
        };

        let text = match self {
            CommandBufferWidget::Success { message } => message.clone().unwrap_or(String::new()),
            CommandBufferWidget::Error { message } => message.to_string(),
            CommandBufferWidget::Idle { buf } => buf.to_string(),
        };

        let block = Block::new()
            .borders(Borders::ALL)
            .title("Command Buffer:")
            .padding(Padding::horizontal(1))
            .fg(color);
        Paragraph::new(format!("$ {text}"))
            .block(block)
            .render(area, buf);
    }
}

impl Default for CommandBufferWidget {
    fn default() -> Self {
        Self::Idle { buf: String::new() }
    }
}

impl CommandBufferWidget {
    pub fn handle_event(&mut self, event: &Event) -> Option<Command> {
        let Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: _,
        }) = event
        else {
            return None;
        };

        use KeyCode::*;
        match code {
            Backspace => {
                self.get_buf().pop();
            }
            Enter => {
                return self.parse_command();
            }
            Char(c) => {
                self.get_buf().push(*c);
            }
            _ => (),
        };
        None
    }

    fn get_buf(&mut self) -> &mut String {
        match self {
            Self::Success { .. } | Self::Error { .. } => {
                *self = Self::default();
            }
            Self::Idle { .. } => (),
        };
        let Self::Idle { buf } = self else {
            unreachable!()
        };
        buf
    }

    fn parse_command(&mut self) -> Option<Command> {
        let buf = self.get_buf().clone();
        if buf.is_empty() {
            return None;
        };
        match parse_command(&buf) {
            Ok((_, command)) => Some(command),
            Err(nom::Err::Error(e)) => {
                self.error(e.to_string());
                None
            }
            Err(nom::Err::Incomplete(_)) => {
                self.error(String::from("Incomplete command"));
                None
            }
            Err(nom::Err::Failure(f)) => {
                self.error(f.to_string());
                None
            }
        }
    }

    pub fn success(&mut self, message: String) {
        *self = Self::Success {
            message: Some(message),
        };
    }

    pub fn error(&mut self, message: String) {
        *self = Self::Error { message };
    }
}
