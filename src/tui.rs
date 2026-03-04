use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::Rect,
    prelude::*,
    widgets::{self, Block, Widget},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use crate::{
    command::Command,
    data_structures::show::Show,
    tui::{
        command_buffer_widget::CommandBufferWidget, logs_widget::LogsWidget,
        tombstones_widget::TombstonesWidget,
    },
};

mod command_buffer_widget;
mod logs_widget;
mod tombstones_widget;

#[derive(EnumIter, Display, PartialEq, Eq, FromRepr, Clone, Copy)]
#[repr(usize)]
enum Tabs {
    Tombstones,
    Setup,
    Logs,
}

pub struct Tui {
    command_buffer: CommandBufferWidget,
    tombstones: TombstonesWidget,
    logs: LogsWidget,
    selected_tab: Tabs,
}

impl StatefulWidget for &mut Tui {
    type State = Show;

    fn render(self, area: Rect, buf: &mut Buffer, show: &mut Show)
    where
        Self: Sized,
    {
        let [header_area, tabs_area, main_area, command_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .areas(area);

        let path = match show.runtime.showfile.as_ref() {
            Some(path) => path.to_string_lossy().to_string(),
            None => "<unsaved>".to_string(),
        };
        Line::raw(path).centered().render(header_area, buf);

        widgets::Tabs::new(Tabs::iter().map(|t| t.to_string()))
            .select(Tabs::iter().position(|t| t == self.selected_tab))
            .render(tabs_area, buf);
        Block::bordered().render(main_area, buf);

        let inner_area = Block::bordered().inner(main_area);

        match self.selected_tab {
            Tabs::Tombstones => self.tombstones.render(inner_area, buf, show),
            Tabs::Logs => self.logs.render(inner_area, buf, show),
            Tabs::Setup => (),
        }

        self.command_buffer.render(command_area, buf);
    }
}

impl Tui {
    pub fn new() -> Self {
        Self {
            command_buffer: CommandBufferWidget::default(),
            tombstones: TombstonesWidget::new(),
            logs: LogsWidget::new(),
            selected_tab: Tabs::Tombstones,
        }
    }

    pub fn handle_event(&mut self, event: Event, show: &Show) -> Option<Command> {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: _,
        }) = event
        {
            return Some(Command::Quit);
        };

        if let Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: _,
        }) = event
        {
            self.cycle_tab();
            return None;
        };

        self.tombstones.handle_event(&event, show);
        self.command_buffer.handle_event(&event)
    }

    fn cycle_tab(&mut self) {
        let selected = self.selected_tab as usize;
        let next = (selected + 1) % Tabs::iter().len();
        self.selected_tab = Tabs::from_repr(next).expect("should always be valid tab discriminant")
    }

    pub fn command_result(&mut self, res: Result<String, String>) {
        match res {
            Ok(success) => self.command_buffer.success(success),
            Err(failure) => self.command_buffer.error(failure),
        }
    }
}

impl Default for Tui {
    fn default() -> Self {
        Self::new()
    }
}
