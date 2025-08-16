use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};

use crate::sdl::EngineContext;

pub struct ElectricalBlock {
    battery_voltage: f32,
    rpm: u16,
}

impl ElectricalBlock {
    pub fn new(ctx: &EngineContext) -> Self {
        Self {
            battery_voltage: ctx.battery_voltage,
            rpm: ctx.engine_speed,
        }
    }
}

impl Widget for ElectricalBlock {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Block::new()
            .borders(Borders::ALL)
            .title(Span::styled(
                "ELECTRICAL",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(area, buf);
        let electrical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), // block hdr
                Constraint::Length(1), // batt volt
                Constraint::Length(1), // block ftr
            ])
            .split(area.inner(Margin::new(1, 0)));
        let battery_color = match self.rpm {
            rpm if rpm > 0 => match self.battery_voltage {
                batt_volt if !(13.1..=15.2).contains(&batt_volt) => (Color::Red, Color::White),
                _ => (Color::Black, Color::White),
            },
            _ => match self.battery_voltage {
                batt_volt if batt_volt <= 12.2 || batt_volt > 13.1 => (Color::Red, Color::White),
                batt_volt if batt_volt < 12.4 || (batt_volt > 12.8 && batt_volt <= 13.1) => {
                    (Color::Black, Color::LightYellow)
                }
                batt_volt if (12.4..=12.8).contains(&batt_volt) => (Color::Black, Color::White),
                _ => (Color::Black, Color::Red),
            },
        };
        Span::styled(
            format!("BATT: {:.1} V", self.battery_voltage),
            Style::default()
                .bg(battery_color.0)
                .fg(battery_color.1)
                .add_modifier(Modifier::BOLD),
        )
        .render(electrical_layout[1], buf);
    }
}
