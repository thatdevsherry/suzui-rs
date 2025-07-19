use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};

use crate::sdl::EngineContext;

pub struct VehicleBlock {
    speed: u8,
    instant_consumption: f32,
}

impl VehicleBlock {
    pub fn new(ctx: &EngineContext) -> Self {
        Self {
            speed: ctx.vehicle_speed,
            instant_consumption: ctx.instant_consumption,
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
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(speed_block[1]);
        let speed_color = match self.speed {
            speed if speed < 60 => (Color::Black, Color::White),
            speed if speed <= 80 => (Color::Black, Color::Green),
            speed if speed <= 110 => (Color::Black, Color::LightYellow),
            _ => (Color::Red, Color::White),
        };
        let speed = Span::styled(
            format!("{} km/h", self.speed),
            Style::default()
                .bg(speed_color.0)
                .fg(speed_color.1)
                .add_modifier(Modifier::BOLD),
        );
        let instant_consumption = Span::styled(
            format!("{:.1} L/100km", self.instant_consumption),
            Style::default().add_modifier(Modifier::BOLD),
        );
        speed.render(row[0], buf);
        instant_consumption.render(row[1], buf);
    }
}
