use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};

use crate::sdl::EngineContext;

pub struct VehicleBlock {
    speed: u8,
}

impl VehicleBlock {
    pub fn new(ctx: &EngineContext) -> Self {
        Self {
            speed: ctx.vehicle_speed,
        }
    }
}

impl Widget for VehicleBlock {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Block::new()
            .borders(Borders::ALL)
            .title(Span::styled(
                "VEHICLE",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(area, buf);
        let speed_block = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area.inner(Margin::new(1, 0)));
        let speed_color = match self.speed {
            speed if speed < 60 => (Color::Black, Color::White),
            speed if speed <= 80 => (Color::Black, Color::Green),
            speed if speed <= 110 => (Color::Black, Color::LightYellow),
            _ => (Color::Red, Color::White),
        };
        let speed = Span::styled(
            format!("Speed: {} km/h", self.speed),
            Style::default()
                .bg(speed_color.0)
                .fg(speed_color.1)
                .add_modifier(Modifier::BOLD),
        );
        speed.render(speed_block[1], buf);
    }
}
