use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
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
                Constraint::Length(1), // data
                Constraint::Length(1),
            ])
            .split(area.inner(Margin::new(1, 0)));
        let flags_layout_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(flags_layout[1]);
        Paragraph::new("E/L")
            .style(
                Style::default()
                    .fg(if self.el { Color::White } else { Color::White })
                    .bg(if self.el { Color::Green } else { Color::Black })
                    .bold(),
            )
            .centered()
            .render(flags_layout_split[0].inner(Margin::new(1, 0)), buf);
        Paragraph::new("A/C")
            .style(
                Style::default()
                    .fg(if self.ac { Color::White } else { Color::White })
                    .bg(if self.ac { Color::Green } else { Color::Black })
                    .bold(),
            )
            .centered()
            .render(flags_layout_split[1].inner(Margin::new(1, 0)), buf);
        Paragraph::new("PSP")
            .style(
                Style::default()
                    .fg(if self.psp { Color::White } else { Color::White })
                    .bg(if self.psp { Color::Green } else { Color::Black })
                    .bold(),
            )
            .centered()
            .render(flags_layout_split[2].inner(Margin::new(1, 0)), buf);
        Paragraph::new("RAD")
            .style(
                Style::default()
                    .fg(if self.rad_fan {
                        Color::White
                    } else {
                        Color::White
                    })
                    .bg(if self.rad_fan {
                        Color::Green
                    } else {
                        Color::Black
                    })
                    .bold(),
            )
            .centered()
            .render(flags_layout_split[3].inner(Margin::new(1, 0)), buf);
    }
}
