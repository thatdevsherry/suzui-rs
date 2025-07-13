use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};

use crate::sdl::EngineContext;

pub struct FlagsBlock {
    el: bool,
    ac: bool,
    psp: bool,
    rad_fan: bool,
}

impl FlagsBlock {
    pub fn new(ctx: &EngineContext) -> Self {
        Self {
            el: ctx.electric_load,
            ac: ctx.ac_switch,
            psp: ctx.psp_switch,
            rad_fan: ctx.radiator_fan,
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
                Constraint::Percentage(100), // data
                Constraint::Length(1),
            ])
            .split(area.inner(Margin::new(1, 0)));
        let flags_layout_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Length(1)])
            .split(flags_layout[1]);
        let flags_layout_split_top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(flags_layout_split[0]);
        let flags_layout_split_bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(flags_layout_split[1]);
        Span::styled(
            "EL:",
            Style::default()
                .fg(if self.el { Color::Green } else { Color::White })
                .add_modifier(Modifier::BOLD),
        )
        .render(flags_layout_split_top[0], buf);
        Span::styled(
            if self.el { "ON" } else { "OFF" },
            Style::default()
                .fg(if self.el { Color::Green } else { Color::White })
                .add_modifier(Modifier::BOLD),
        )
        .render(flags_layout_split_top[1], buf);
        Span::styled(
            "AC:",
            Style::default()
                .fg(if self.ac { Color::Green } else { Color::White })
                .add_modifier(Modifier::BOLD),
        )
        .render(flags_layout_split_top[2], buf);
        Span::styled(
            if self.ac { "ON" } else { "OFF" },
            Style::default()
                .fg(if self.ac { Color::Green } else { Color::White })
                .add_modifier(Modifier::BOLD),
        )
        .render(flags_layout_split_top[3], buf);
        Span::styled(
            "PSP:",
            Style::default()
                .fg(if self.psp { Color::Green } else { Color::White })
                .add_modifier(Modifier::BOLD),
        )
        .render(flags_layout_split_bottom[0], buf);
        Span::styled(
            if self.psp { "ON" } else { "OFF" },
            Style::default()
                .fg(if self.psp { Color::Green } else { Color::White })
                .add_modifier(Modifier::BOLD),
        )
        .render(flags_layout_split_bottom[1], buf);
        Span::styled(
            "RAD:",
            Style::default()
                .fg(if self.psp { Color::Green } else { Color::White })
                .add_modifier(Modifier::BOLD),
        )
        .render(flags_layout_split_bottom[2], buf);
        Span::styled(
            if self.rad_fan { "ON" } else { "OFF" },
            Style::default()
                .fg(if self.psp { Color::Green } else { Color::White })
                .add_modifier(Modifier::BOLD),
        )
        .render(flags_layout_split_bottom[3], buf);
    }
}
