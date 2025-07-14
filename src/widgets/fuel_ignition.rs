use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge},
};

use crate::sdl::EngineContext;

pub struct FuelIgnitionBlock {
    inj_pw: f32,
    fuel_cut: bool,
    ignition_advance: i8,
}

impl FuelIgnitionBlock {
    pub fn new(ctx: &EngineContext) -> Self {
        Self {
            inj_pw: ctx.injector_pulse_width_cyl_1,
            fuel_cut: ctx.fuel_cut,
            ignition_advance: ctx.ignition_advance,
        }
    }
}

impl Widget for FuelIgnitionBlock {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Block::new()
            .borders(Borders::ALL)
            .title(Span::styled(
                "FUEL/IGNITION",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(area, buf);
        let fuel_ignition_block_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), // block header
                Constraint::Length(3), // inj pw
                Constraint::Length(1), // ign adv
                Constraint::Length(1), // block footer
            ])
            .split(area.inner(Margin::new(1, 0)));
        if self.fuel_cut {
            Gauge::default()
                .percent(100)
                .gauge_style(Style::default().fg(Color::Green))
                .label(Span::styled(
                    "FUEL CUT",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ))
                .render(fuel_ignition_block_layout[1], buf);
        } else {
            let inj_pw_percentage = ((self.inj_pw / 20.0_f32) * 100.0).min(100.0) as u16;
            Gauge::default()
                .percent(inj_pw_percentage)
                .gauge_style(Style::default().fg(Color::White))
                .label(Span::styled(
                    format!("{:.1} ms", self.inj_pw),
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ))
                .render(fuel_ignition_block_layout[1], buf);
        }
        Span::styled(
            format!("IGN ADV: {}", self.ignition_advance),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .render(fuel_ignition_block_layout[2], buf);
    }
}
