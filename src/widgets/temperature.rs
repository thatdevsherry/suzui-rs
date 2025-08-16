use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge},
};

use crate::sdl::EngineContext;

#[derive(Debug)]
pub struct TemperatureState {
    is_iat_red: bool,
    is_ect_red: bool,
}

impl TemperatureState {
    pub fn new() -> Self {
        Self {
            is_iat_red: false,
            is_ect_red: false,
        }
    }
}

pub struct TemperatureBlock {
    coolant: i8,
    intake: i8,
}

impl TemperatureBlock {
    pub fn new(ctx: &EngineContext) -> Self {
        Self {
            coolant: ctx.coolant_temp,
            intake: ctx.intake_air_temperature,
        }
    }
}

impl StatefulWidget for TemperatureBlock {
    type State = TemperatureState;

    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        Block::new()
            .borders(Borders::ALL)
            .title(Span::styled(
                "TEMPERATURES",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(area, buf);
        let temperature_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), // block hdr
                Constraint::Length(1), // intake
                Constraint::Length(3), // coolant
                Constraint::Length(1), // block ftr
            ])
            .split(area.inner(Margin::new(1, 0)));
        let coolant_temp_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(10), Constraint::Percentage(100)])
            .split(temperature_layout[2]);
        let intake_temp_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(10), Constraint::Percentage(100)])
            .split(temperature_layout[1]);
        let coolant_temp_layout_text = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(coolant_temp_layout[0]);
        let intake_temp_layout_text = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(intake_temp_layout[0]);
        Span::styled(
            "ECT:",
            Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .render(coolant_temp_layout_text[1], buf);
        let coolant_min = 60;
        let coolant_max = 120;
        let coolant_percentage = if self.coolant <= coolant_min {
            0
        } else if self.coolant >= coolant_max {
            100
        } else {
            ((self.coolant - coolant_min) as f32 / (coolant_max - coolant_min) as f32 * 100.0)
                as u16
        };
        let coolant_color = match self.coolant {
            temp if temp < 85 => Color::Blue,
            temp if temp < 101 => Color::Green,
            temp if temp < 110 => Color::LightYellow,
            _ => {
                let color;
                if state.is_ect_red {
                    state.is_ect_red = false;
                    color = Color::Black
                } else {
                    state.is_ect_red = true;
                    color = Color::Red
                }
                color
            }
        };
        Gauge::default()
            .percent(coolant_percentage)
            .gauge_style(Style::default().fg(coolant_color))
            .label(Span::styled(
                format!("{} °C", self.coolant),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(coolant_temp_layout[1], buf);
        Span::styled(
            "IAT:",
            Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .render(intake_temp_layout_text[1], buf);
        let intake_min = 0;
        let intake_max = 80;
        let intake_percentage = if self.intake <= intake_min {
            0
        } else if self.intake >= intake_max {
            100
        } else {
            ((self.intake - intake_min) as f64 / (intake_max - intake_min) as f64 * 100.0) as u16
        };
        let intake_color = match self.intake {
            iat if iat < 20 => Color::Blue,
            iat if iat < 50 => Color::Green,
            iat if iat < 70 => Color::Yellow,
            _ => {
                let color;
                if state.is_iat_red {
                    state.is_iat_red = false;
                    color = Color::Black
                } else {
                    state.is_iat_red = true;
                    color = Color::Red;
                }
                color
            }
        };
        Gauge::default()
            .percent(intake_percentage)
            .gauge_style(Style::default().fg(intake_color))
            .label(Span::styled(
                format!("{} °C", self.intake),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(intake_temp_layout[1], buf);
    }
}
