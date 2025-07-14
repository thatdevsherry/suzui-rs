use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge},
};

use crate::sdl::EngineContext;

pub struct AirflowBlock {
    map: f32,
    baro: f32,
    calc_load: u8,
}

impl AirflowBlock {
    pub fn new(ctx: &EngineContext) -> Self {
        Self {
            map: ctx.manifold_absolute_pressure,
            baro: ctx.barometric_pressure,
            calc_load: ctx.calculated_load,
        }
    }
}

impl Widget for AirflowBlock {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Block::new()
            .borders(Borders::ALL)
            .title(Span::styled(
                "LOAD",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(area, buf);
        let airflow_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), // block hdr
                Constraint::Length(1), // map + baro
                Constraint::Length(1), // load
                Constraint::Length(1), // block ftr
            ])
            .split(area.inner(Margin::new(1, 0)));
        Span::styled(
            format!("MAP: {:.2} ({:.2})", self.map, self.baro),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .render(airflow_layout[1], buf);
        let gauge_color = match self.calc_load {
            _ => Color::White,
        };
        let calc_load_min = 0;
        let calc_load_max = 100;
        let calc_load_percentage = if self.calc_load <= calc_load_min {
            0
        } else if self.calc_load >= calc_load_max {
            100
        } else {
            ((self.calc_load - calc_load_min) as f32 / (calc_load_max - calc_load_min) as f32
                * 100.0) as u16
        };
        Gauge::default()
            .percent(calc_load_percentage)
            .gauge_style(Style::default().fg(gauge_color))
            .label(Span::styled(
                format!("{} %", self.calc_load),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(airflow_layout[2], buf);
    }
}
