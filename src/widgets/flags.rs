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
            .white()
            .bg(if self.el { Color::Green } else { Color::Black })
            .centered()
            .bold()
            .render(flags_layout_split[0], buf);
        Paragraph::new("A/C")
            .white()
            .bg(if self.ac { Color::Green } else { Color::Black })
            .bold()
            .centered()
            .render(flags_layout_split[1], buf);
        Paragraph::new("PSP")
            .white()
            .bold()
            .bg(if self.psp { Color::Green } else { Color::Black })
            .centered()
            .render(flags_layout_split[2], buf);
        Paragraph::new("RAD")
            .white()
            .bold()
            .bg(if self.rad_fan {
                Color::Green
            } else {
                Color::Black
            })
            .centered()
            .render(flags_layout_split[3], buf);
    }
}
