use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph},
    DefaultTerminal, Frame,
};
use strum::IntoEnumIterator;
use suzui_rs::sdl::{ObdAddress, ScanToolParameter, SuzukiSdlViewer};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    //sdl_viewer: SuzukiSdlViewer,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        //self.sdl_viewer.connect();
        while self.running {
            //self.sdl_viewer.update_raw_data();
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        // Initial layout
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Ratio(4, 5), Constraint::Ratio(1, 5)])
            .split(frame.area());
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[0]);
        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(top[0]);
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(top[1]);
        frame.render_widget(Block::new().borders(Borders::ALL), layout[0]);
        frame.render_widget(
            Block::new().borders(Borders::ALL).title("Status"),
            layout[1],
        );
        frame.render_widget(
            Block::new().borders(Borders::ALL).title("Engine speed"),
            left[0],
        );
        frame.render_widget(
            Block::new().borders(Borders::ALL).title("Fuel/Ignition"),
            left[1],
        );
        frame.render_widget(
            Block::new().borders(Borders::ALL).title("Temperatures"),
            left[2],
        );
        frame.render_widget(
            Block::new().borders(Borders::ALL).title("Throttle"),
            right[0],
        );
        frame.render_widget(
            Block::new().borders(Borders::ALL).title("Airflow"),
            right[1],
        );
        frame.render_widget(
            Block::new().borders(Borders::ALL).title("Electrical"),
            right[2],
        );

        // Engine speed block
        let engine_speed_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Ratio(2, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(left[0].inner(Margin::new(1, 0)));
        let rpm_gauge = Gauge::default()
            .percent(((3250 as f64 / 6500 as f64) * 100.0).min(100.0) as u16)
            .label(format!("{} RPM", 3250));
        frame.render_widget(rpm_gauge, engine_speed_layout[1]);
        let rpm_misc_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(engine_speed_layout[2]);
        frame.render_widget(Paragraph::new("Target: 0000"), rpm_misc_layout[0]);
        frame.render_widget(Paragraph::new("ISC: 000 %"), rpm_misc_layout[1]);

        // Throttle block
        let throttle_block_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Ratio(2, 4),
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
            ])
            .split(right[0].inner(Margin::new(1, 0)));
        let throttle_value = ((128 as f64 / 255 as f64) * 100.0).round();
        let throttle_gauge = Gauge::default()
            .percent(throttle_value as u16)
            .label(format!("{} %", throttle_value));
        frame.render_widget(throttle_gauge, throttle_block_layout[1]);
        frame.render_widget(
            Paragraph::new("TPS Voltage: 5.00 V"),
            throttle_block_layout[2],
        );
        frame.render_widget(Paragraph::new("CTP: OFF"), throttle_block_layout[3]);

        // Fuel/Ignition block
        let fuel_ignition_block_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(left[1].inner(Margin::new(1, 0)));
        frame.render_widget(
            Paragraph::new("INJ PW: 131.07 ms"),
            fuel_ignition_block_layout[1],
        );
        frame.render_widget(
            Paragraph::new("Fuel Cut: OFF"),
            fuel_ignition_block_layout[2],
        );
        frame.render_widget(
            Paragraph::new("IGN ADV: -12"),
            fuel_ignition_block_layout[3],
        );

        // Airflow block
        let airflow_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
            ])
            .split(right[1].inner(Margin::new(1, 0)));
        frame.render_widget(Paragraph::new("MAP: 146.63 kPa"), airflow_layout[1]);
        frame.render_widget(Paragraph::new("Baro: 146.63 kPa"), airflow_layout[2]);

        // Temperature block
        let temperature_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(left[2].inner(Margin::new(1, 0)));
        let coolant_temp_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
            .split(temperature_layout[1]);
        let intake_temp_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
            .split(temperature_layout[2]);
        frame.render_widget(Paragraph::new("Coolant:"), coolant_temp_layout[0]);
        let coolant_min = 70;
        let coolant_max = 110;
        let coolant_current = 108;
        let coolant_percentage = if coolant_current <= coolant_min {
            0
        } else if coolant_current >= coolant_max {
            100
        } else {
            ((coolant_current - coolant_min) as f64 / (coolant_max - coolant_min) as f64 * 100.0)
                as u16
        };
        let coolant_color = if coolant_current >= 105 {
            Color::Red
        } else if coolant_current >= 98 {
            Color::Yellow
        } else if coolant_current >= 88 {
            Color::Green
        } else {
            Color::Blue
        };
        let coolant_gauge = Gauge::default()
            .percent(coolant_percentage)
            .gauge_style(Style::default().fg(coolant_color))
            .label(Span::styled(
                format!("{} C", coolant_current),
                Style::default().fg(Color::White).bg(Color::Black),
            ));
        frame.render_widget(coolant_gauge, coolant_temp_layout[1]);
        frame.render_widget(Paragraph::new("Intake:"), intake_temp_layout[0]);
        let intake_min = -40;
        let intake_max = 70;
        let intake_current = 50;
        let intake_percentage = if intake_current <= intake_min {
            0
        } else if intake_current >= intake_max {
            100
        } else {
            ((intake_current - intake_min) as f64 / (intake_max - intake_min) as f64 * 100.0) as u16
        };
        let intake_color = if intake_current >= 60 {
            Color::Red
        } else if intake_current >= 45 {
            Color::Yellow
        } else if intake_current >= 0 {
            Color::Green
        } else {
            Color::Cyan
        };
        let intake_gauge = Gauge::default()
            .percent(intake_percentage)
            .gauge_style(Style::default().fg(intake_color))
            .label(Span::styled(
                format!("{} C", intake_current),
                Style::default().fg(Color::White).bg(Color::Black),
            ));
        frame.render_widget(intake_gauge, intake_temp_layout[1]);

        // Electrical block
        let electrical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Length(1)])
            .split(right[2].inner(Margin::new(1, 0)));
        frame.render_widget(
            Paragraph::new("battery voltage: 20.0 V"),
            electrical_layout[1],
        );

        // Status block
        let status_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), // header
                Constraint::Length(1), // speed
                Constraint::Length(1), // flags
            ])
            .split(layout[1].inner(Margin::new(1, 0)));
        frame.render_widget(Paragraph::new("Speed: 255 km/h"), status_layout[1]);
        let flags_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Ratio(1, 4), // EL
                Constraint::Ratio(1, 4), // AC
                Constraint::Ratio(1, 4), // PSP
                Constraint::Ratio(1, 4), // RAD
            ])
            .split(status_layout[2]);
        // EL
        frame.render_widget(Paragraph::new("EL: OFF"), flags_layout[0]);
        // AC
        frame.render_widget(Paragraph::new("AC: OFF"), flags_layout[1]);
        // PSP
        frame.render_widget(Paragraph::new("PSP: OFF"), flags_layout[2]);
        // FAN
        frame.render_widget(Paragraph::new("FAN: OFF"), flags_layout[3]);

        // raw data display
        /*
        for (idx, addr) in ObdAddress::iter().enumerate() {
            let value = self.sdl_viewer.raw_data.get(&addr).unwrap();
            let area = Rect::new(0, idx as u16, addr.to_string().len() as u16, 1);
            let area_value = Rect::new(30, idx as u16, 10, 1);
            frame.render_widget(Paragraph::new(format!("{}:", addr)), area);
            frame.render_widget(Paragraph::new(value.to_string()), area_value);
        }
        */
        // scan tool data display
        /*
        for (idx, scan_tool_parameter) in ScanToolParameter::iter().enumerate() {
            let value = self
                .sdl_viewer
                .scan_tool_data
                .get(&scan_tool_parameter)
                .unwrap();
            let area = Rect::new(
                0,
                idx as u16,
                scan_tool_parameter.to_string().len() as u16,
                1,
            );
            let area_value = Rect::new(30, idx as u16, 10, 1);
            frame.render_widget(Paragraph::new(format!("{}:", scan_tool_parameter)), area);
            frame.render_widget(Paragraph::new(value.to_string()), area_value);
        }
        */
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
