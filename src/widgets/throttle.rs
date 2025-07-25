use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge},
};

use crate::sdl::EngineContext;

pub struct ThrottleBlock {
    abs_throttle_position: u8,
    angle: u8,
    ctp: bool,
}

impl ThrottleBlock {
    pub fn new(ctx: &EngineContext) -> Self {
        Self {
            abs_throttle_position: ctx.absolute_throttle_position,
            angle: ctx.throttle_angle,
            ctp: ctx.closed_throttle_position,
        }
    }
}

impl Widget for ThrottleBlock {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Block::new()
            .borders(Borders::ALL)
            .title(Span::styled(
                "THROTTLE",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(area, buf);
        // Throttle block
        let throttle_block_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), // block hdr
                Constraint::Length(5), // throttle
                Constraint::Length(1), // block ftr
            ])
            .split(area.inner(Margin::new(1, 0)));
        Gauge::default()
            .percent(self.abs_throttle_position as u16)
            .gauge_style(Style::default().fg(match self.angle {
                angle if angle >= 80 => Color::Blue,
                _ => match self.ctp {
                    true => Color::Green,
                    false => Color::White,
                },
            }))
            .label(Span::styled(
                format!("{}°", self.angle),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(throttle_block_layout[1], buf);
    }
}
