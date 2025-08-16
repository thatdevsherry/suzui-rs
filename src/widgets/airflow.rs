use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge},
};

use crate::sdl::EngineContext;

pub struct AirflowBlock {
    map: f32,
    baro: f32,
    calc_load: u8,
    rpm: u16,
}

impl AirflowBlock {
    pub fn new(ctx: &EngineContext) -> Self {
        Self {
            map: ctx.manifold_absolute_pressure,
            baro: ctx.barometric_pressure,
            calc_load: ctx.calculated_load,
            rpm: ctx.engine_speed,
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

        // doing this as load can go over 100% in certain cases.
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
        let gauge_color = match self.rpm {
            rpm if rpm < 2000 && self.calc_load >= 85 => Color::Red,
            _ => Color::White,
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
