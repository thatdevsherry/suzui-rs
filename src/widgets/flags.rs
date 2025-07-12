use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};

use crate::sdl::EngineContext;

pub struct FlagsBlock {
    el: bool,
    ac: bool,
    psp: bool,
}

impl FlagsBlock {
    pub fn new(ctx: &EngineContext) -> Self {
        Self {
            el: ctx.electric_load,
            ac: ctx.ac_switch,
            psp: ctx.psp_switch,
        }
    }
}

impl Widget for FlagsBlock {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Block::new()
            .borders(Borders::ALL)
            .title(Span::styled(
                "STATUS FLAGS",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(area, buf);
        let flags_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1), // EL
                Constraint::Length(1), // AC
                Constraint::Length(1), // PSP
                Constraint::Length(1),
            ])
            .split(area.inner(Margin::new(1, 0)));
        Span::styled(
            format!("EL: {}", if self.el { "ON" } else { "OFF" }),
            Style::default()
                .fg(if self.el { Color::Green } else { Color::White })
                .add_modifier(Modifier::BOLD),
        )
        .render(flags_layout[1], buf);
        Span::styled(
            format!("AC: {}", if self.ac { "ON" } else { "OFF" }),
            Style::default()
                .fg(if self.ac { Color::Green } else { Color::White })
                .add_modifier(Modifier::BOLD),
        )
        .render(flags_layout[2], buf);
        Span::styled(
            format!("PSP: {}", if self.psp { "ON" } else { "OFF" }),
            Style::default()
                .fg(if self.psp { Color::Green } else { Color::White })
                .add_modifier(Modifier::BOLD),
        )
        .render(flags_layout[3], buf);
    }
}
