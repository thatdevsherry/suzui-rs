use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge},
};

use crate::sdl::EngineContext;

pub struct EngineSpeedBlock {
    rpm: u16,
    desired_idle: u16,
    isc: u8,
}

impl EngineSpeedBlock {
    pub fn new(ctx: &EngineContext) -> Self {
        Self {
            rpm: ctx.engine_speed,
            desired_idle: ctx.desired_idle,
            isc: ctx.isc_flow_duty,
        }
    }
}

impl Widget for EngineSpeedBlock {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Block::new()
            .borders(Borders::ALL)
            .title(Span::styled(
                "ENGINE SPEED",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(area, buf);
        let engine_speed_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), // header
                Constraint::Length(5), // rpm
                Constraint::Length(1), // desired idle
                Constraint::Length(1), // isc flow duty
                Constraint::Length(1), // footer
            ])
            .split(area.inner(Margin::new(1, 0)));
        let engine_rpm_percentage = ((self.rpm as f64 / 6500_f64) * 100.0).min(100.0) as u16;
        let engine_rpm_color = match self.rpm {
            rpm if rpm < 500 => Color::Red,
            rpm if rpm < 2500 => Color::White,
            rpm if rpm < 5500 => Color::Green,
            rpm if rpm < 6000 => Color::LightYellow,
            _ => Color::Red,
        };
        Gauge::default()
            .percent(engine_rpm_percentage)
            .gauge_style(Style::default().fg(engine_rpm_color))
            .label(Span::styled(
                format!("{} RPM", ((self.rpm + 25) / 50) * 50),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(engine_speed_layout[1], buf);
        Span::styled(
            format!("IDLE: {} RPM", self.desired_idle),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .render(engine_speed_layout[2], buf);
        Gauge::default()
            .percent(self.isc as u16)
            .gauge_style(Style::default().fg(Color::White))
            .label(Span::styled(
                format!("{} %", self.isc),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(engine_speed_layout[3], buf);
    }
}
