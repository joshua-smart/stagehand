use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind::Press, KeyModifiers},
    prelude::*,
    widgets::{Block, Scrollbar, ScrollbarState, Widget},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator as _};

use crate::data_structures::{address::Address, index::Index, level::Level, show::Show};

#[derive(Display, EnumIter, FromRepr, Clone, Copy)]
#[repr(u8)]
enum Filter {
    All,
    Active,
}

#[derive(Display, EnumIter, FromRepr, Clone, Copy)]
#[repr(u8)]
enum Mode {
    Decimal,
    Percentage,
}

pub struct TombstonesWidget {
    universe_index: usize,
    mode: Mode,
    filter: Filter,
    scroll: usize,
}

impl TombstonesWidget {
    pub fn new() -> Self {
        Self {
            universe_index: 0,
            mode: Mode::Decimal,
            filter: Filter::Active,
            scroll: 0,
        }
    }

    fn draw_controls(&self, show: &Show, area: Rect, buf: &mut Buffer) {
        let universe_control = Line::from(vec![
            "Universe: ".into(),
            "◄F1 ".fg(Color::Gray),
            show.universes
                .iter()
                .nth(self.universe_index)
                .map(|u| u.to_string())
                .unwrap_or(String::from("-"))
                .into(),
            " F2►".fg(Color::Gray),
        ]);

        let filter_control = Line::from(vec![
            "Filter: ".into(),
            self.filter.to_string().into(),
            " F3▲".fg(Color::Gray),
        ]);

        let mode_control = Line::from(vec![
            "Mode: ".into(),
            self.mode.to_string().into(),
            " F4▲".fg(Color::Gray),
        ]);

        let [
            universe_control_area,
            filter_control_area,
            mode_control_area,
        ] = Layout::horizontal([
            Constraint::Length(universe_control.width() as u16),
            Constraint::Length(filter_control.width() as u16),
            Constraint::Length(mode_control.width() as u16),
        ])
        .spacing(3)
        .areas(area);

        universe_control.render(universe_control_area, buf);
        filter_control.render(filter_control_area, buf);
        mode_control.render(mode_control_area, buf);
    }

    fn draw_stones(
        &self,
        levels: Vec<(Index, Option<Level>)>,
        rows: usize,
        columns: usize,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let vlayout = Layout::new(Direction::Vertical, vec![5; rows]).split(area);

        'outer: for row in 0..rows {
            let area = vlayout[row];

            let hlayout = Layout::new(Direction::Horizontal, vec![5; columns]).split(area);

            for col in 0..columns {
                let area = hlayout[col];

                let Some((index, level)) = levels.get(self.scroll * columns + col + row * columns)
                else {
                    break 'outer;
                };
                let layout = Layout::new(Direction::Vertical, [1, 1, 1])
                    .split(Block::bordered().inner(area));

                let level_fmt = level
                    .map(|l| match self.mode {
                        Mode::Decimal => l.to_string(),
                        Mode::Percentage => {
                            let p = u8::from(l) as f64 * 100_f64 / 255_f64;
                            format!("{p:.0}%")
                        }
                    })
                    .unwrap_or(String::from("-"));

                Block::bordered().render(area, buf);
                Line::raw(level_fmt)
                    .centered()
                    .bold()
                    .fg(match level.map(u8::from) {
                        Some(u8::MIN) => Color::Red,
                        Some(u8::MAX) => Color::Green,
                        Some(_) => Color::Blue,
                        None => Color::Gray,
                    })
                    .render(layout[0], buf);
                Line::raw(index.to_string())
                    .centered()
                    .render(layout[2], buf);
            }
        }
    }

    pub fn handle_event(&mut self, event: &Event, show: &Show) {
        let Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: Press,
            state: _,
        }) = event
        else {
            return;
        };

        match code {
            KeyCode::PageDown => {
                self.scroll = self.scroll.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::F(1) => {
                let n = show.universes.len();
                self.universe_index = (self.universe_index + n - 1) % show.universes.len();
            }
            KeyCode::F(2) => {
                self.universe_index = (self.universe_index + 1) % show.universes.len();
            }
            KeyCode::F(3) => {
                self.filter =
                    Filter::from_repr((self.filter as u8 + 1) % Filter::iter().len() as u8)
                        .unwrap();
            }
            KeyCode::F(4) => {
                self.mode =
                    Mode::from_repr((self.mode as u8 + 1) % Mode::iter().len() as u8).unwrap();
            }
            _ => (),
        }
    }
}

impl Default for TombstonesWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl StatefulWidget for &mut TombstonesWidget {
    type State = Show;

    fn render(self, area: Rect, buf: &mut Buffer, show: &mut Self::State) {
        let set_addresses = &show.runtime.set_addresses;

        let universe = show.universes.iter().nth(self.universe_index).unwrap();

        let levels = match self.filter {
            Filter::All => Index::range()
                .map(|i| {
                    (
                        i,
                        set_addresses
                            .get(&Address {
                                universe: *universe,
                                index: i,
                            })
                            .cloned(),
                    )
                })
                .collect::<Vec<_>>(),
            Filter::Active => set_addresses
                .iter()
                .filter(
                    |(
                        Address {
                            universe: u,
                            index: _,
                        },
                        _,
                    )| u == universe,
                )
                .map(|(Address { universe: _, index }, level)| (*index, Some(level).cloned()))
                .collect::<Vec<_>>(),
        };

        let columns = area.width as usize / 5;
        let page_rows = area.height as usize / 5;

        let max_rows = (levels.len()).div_ceil(columns);
        let max_scroll = max_rows.saturating_sub(page_rows);
        self.scroll = self.scroll.clamp(0, max_scroll);

        let [header_area, v_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        let [stones_area, scroll_area] = Layout::new(
            Direction::Horizontal,
            [Constraint::Fill(1), Constraint::Length(1)],
        )
        .areas(v_area);

        self.draw_controls(show, header_area, buf);

        self.draw_stones(levels, page_rows, columns, stones_area, buf);

        Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight).render(
            scroll_area,
            buf,
            &mut ScrollbarState::new(max_scroll + 1)
                .position(self.scroll)
                .viewport_content_length(5),
        );
    }
}
