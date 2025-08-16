use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::sdl::EngineContext;

pub struct VehicleBlock {
    speed: u8,
    instant_consumption: f64,
    fuel_consumption: f64,
    distance_travelled: f64,
}

impl VehicleBlock {
    pub fn new(ctx: &EngineContext) -> Self {
        Self {
            speed: ctx.vehicle_speed,
            instant_consumption: ctx.instant_consumption,
            fuel_consumption: ctx.fuel_consumption,
            distance_travelled: ctx.cumulative_distance,
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
                Constraint::Length(1), // row 1
                Constraint::Length(1), // row 2
                Constraint::Length(1),
            ])
            .split(area.inner(Margin::new(1, 0)));
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100), Constraint::Length(5)])
            .split(speed_block[1]);
        let row_two = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100), Constraint::Length(5)])
            .split(speed_block[2]);
        let speed_color = match self.speed {
            speed if speed <= 120 => (Color::Black, Color::White),
            _ => (Color::Red, Color::White),
        };
        let speed = Paragraph::new(self.speed.to_string())
            .style(Style::default().bg(speed_color.0).fg(speed_color.1))
            .centered()
            .bold();
        let speed_unit = Paragraph::new("kph")
            .style(Style::default().bg(speed_color.0).fg(speed_color.1))
            .centered()
            .bold();
        let fuel_consumption = Paragraph::new(format!(
            "FC: {:.1} ({:.1})",
            self.fuel_consumption, self.instant_consumption
        ))
        .bold()
        .white();
        let odo = Paragraph::new(format!("ODO: {:.1} km", self.distance_travelled))
            .bold()
            .white();
        fuel_consumption.render(row_two[0], buf);
        odo.render(row[0], buf);
        speed.render(row[1], buf);
        speed_unit.render(row_two[1], buf);
    }
}
