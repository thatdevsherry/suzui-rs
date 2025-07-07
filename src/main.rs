use std::{thread::sleep, time::Duration};

use color_eyre::Result;
use crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
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
    sdl_viewer: SuzukiSdlViewer,
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
            self.sdl_viewer.update_raw_data();
            self.sdl_viewer.update_processed_data();
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
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());
        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(layout[0]);
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(layout[1]);
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
        frame.render_widget(Block::new().borders(Borders::ALL).title("Vehicle"), left[3]);
        frame.render_widget(
            Block::new().borders(Borders::ALL).title("Status Flags"),
            right[3],
        );

        // Engine speed block
        let engine_speed_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Max(3),
                Constraint::Length(1),
            ])
            .split(left[0].inner(Margin::new(1, 0)));
        let engine_rpm_current = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::EngineSpeed)
            .unwrap();
        let desired_idle = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::DesiredIdle)
            .unwrap();
        let isc_flow_duty = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::IacFlowDutyCycle)
            .unwrap();
        let engine_rpm_percentage =
            ((engine_rpm_current.value as f64 / 6500 as f64) * 100.0).min(100.0) as u16;
        let engine_rpm_color = match engine_rpm_current.value {
            rpm if rpm < 500.0 => Color::Red,
            rpm if rpm < 2500.0 => Color::White,
            rpm if rpm < 4500.0 => Color::Green,
            rpm if rpm < 6500.0 => Color::Yellow,
            _ => Color::Red,
        };
        let rpm_gauge = Gauge::default()
            .percent(engine_rpm_percentage)
            .gauge_style(Style::default().fg(engine_rpm_color))
            .label(Span::styled(
                format!("{} RPM", engine_rpm_current.value),
                Style::default().fg(Color::White).bg(Color::Black),
            ));
        frame.render_widget(rpm_gauge, engine_speed_layout[1]);
        let rpm_misc_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(engine_speed_layout[2]);
        frame.render_widget(
            Paragraph::new(format!("Target: {} RPM", desired_idle.value)),
            rpm_misc_layout[0],
        );
        let isc_gauge = Gauge::default()
            .percent(isc_flow_duty.value as u16)
            .label(format!("{} %", isc_flow_duty.value));
        frame.render_widget(isc_gauge, rpm_misc_layout[1]);

        // Throttle block
        let throttle_block_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Max(3),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(right[0].inner(Margin::new(1, 0)));
        let throttle_value = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::AbsoluteThrottlePosition)
            .unwrap();
        let tps_voltage = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::TpSensorVolt)
            .unwrap();
        let ctp = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::ClosedThrottlePos)
            .unwrap();
        let throttle_gauge = Gauge::default()
            .percent(throttle_value.value as u16)
            .label(format!("{} %", throttle_value.value));
        frame.render_widget(throttle_gauge, throttle_block_layout[1]);
        frame.render_widget(
            Paragraph::new(format!("TPS Voltage: {:.2} V", tps_voltage.value)),
            throttle_block_layout[2],
        );
        frame.render_widget(
            Paragraph::new(format!(
                "CTP: {}",
                if ctp.value == 0.0 { "OFF" } else { "ON" }
            )),
            throttle_block_layout[3],
        );

        // Fuel/Ignition block
        let fuel_ignition_block_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Max(3),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(left[1].inner(Margin::new(1, 0)));
        let inj_pw_current = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::InjPulseWidthCyl1)
            .unwrap();
        let fuel_cut = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::FuelCut)
            .unwrap();
        let ignition_advance = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::IgnitionAdvance)
            .unwrap();
        let inj_pw_percentage = ((inj_pw_current.value / 50.0 as f32) * 100.0).min(100.0) as u16;
        let inj_pw_gauge = Gauge::default()
            .percent(inj_pw_percentage)
            //.gauge_style(Style::default().fg(engine_rpm_color))
            .label(Span::styled(
                format!("{:.1} ms", inj_pw_current.value),
                Style::default().fg(Color::White).bg(Color::Black),
            ));
        frame.render_widget(inj_pw_gauge, fuel_ignition_block_layout[1]);
        frame.render_widget(
            Paragraph::new(format!(
                "Fuel Cut: {}",
                if fuel_cut.value == 0.0 { "OFF" } else { "ON" }
            )),
            fuel_ignition_block_layout[2],
        );
        frame.render_widget(
            Paragraph::new(format!("IGN ADV: {}", ignition_advance.value)),
            fuel_ignition_block_layout[3],
        );

        // Airflow block
        let airflow_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(right[1].inner(Margin::new(1, 0)));
        let map = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::Map)
            .unwrap();
        let baro = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::BarometricPressure)
            .unwrap();
        frame.render_widget(
            Paragraph::new(format!("MAP: {:.2} kPa", map.value)),
            airflow_layout[1],
        );
        frame.render_widget(
            Paragraph::new(format!("Baro: {:.2} kPa", baro.value)),
            airflow_layout[2],
        );

        // Temperature block
        let temperature_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Length(1),
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
        let coolant = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::CoolantTemp)
            .unwrap();
        let coolant_min = 60.0;
        let coolant_max = 120.0;
        let coolant_current = coolant.value;
        let coolant_percentage = if coolant_current <= coolant_min {
            0
        } else if coolant_current >= coolant_max {
            100
        } else {
            ((coolant_current - coolant_min) as f32 / (coolant_max - coolant_min) as f32 * 100.0)
                as u16
        };
        let coolant_color = match coolant_current {
            temp if temp < 65.0 => Color::Blue,
            temp if temp < 87.0 => Color::Cyan,
            temp if temp < 93.0 => Color::Green,
            temp if temp < 100.0 => Color::Yellow,
            _ => Color::Red,
        };
        let coolant_gauge = Gauge::default()
            .percent(coolant_percentage)
            .gauge_style(Style::default().fg(coolant_color))
            .label(Span::styled(
                format!("{} °C", coolant_current),
                Style::default().fg(Color::White).bg(Color::Black),
            ));
        frame.render_widget(coolant_gauge, coolant_temp_layout[1]);
        frame.render_widget(Paragraph::new("Intake:"), intake_temp_layout[0]);
        let intake_min = -40.0;
        let intake_max = 70.0;
        let intake = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::CoolantTemp)
            .unwrap();
        let intake_current = intake.value;
        let intake_percentage = if intake_current <= intake_min {
            0
        } else if intake_current >= intake_max {
            100
        } else {
            ((intake_current - intake_min) as f64 / (intake_max - intake_min) as f64 * 100.0) as u16
        };
        let intake_color = if intake_current >= 60.0 {
            Color::Red
        } else if intake_current >= 45.0 {
            Color::Yellow
        } else if intake_current >= 0.0 {
            Color::Green
        } else {
            Color::Cyan
        };
        let intake_gauge = Gauge::default()
            .percent(intake_percentage)
            .gauge_style(Style::default().fg(intake_color))
            .label(Span::styled(
                format!("{} °C", intake_current),
                Style::default().fg(Color::White).bg(Color::Black),
            ));
        frame.render_widget(intake_gauge, intake_temp_layout[1]);

        // Electrical block
        let electrical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Length(1)])
            .split(right[2].inner(Margin::new(1, 0)));
        let battery_voltage = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::BatteryVoltage)
            .unwrap();
        frame.render_widget(
            Paragraph::new(format!("battery voltage: {:.1} V", battery_voltage.value)),
            electrical_layout[1],
        );

        // Speed block
        let speed_block = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Max(3)])
            .split(left[3].inner(Margin::new(1, 0)));
        let speed = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::VehicleSpeed)
            .unwrap();
        let speed_gauge = Gauge::default()
            .percent(((speed.value / 255.0) * 100.0) as u16)
            .label(format!("{} km/h", speed.value));
        frame.render_widget(speed_gauge, speed_block[1]);

        // Status block
        let flags_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1), // EL
                Constraint::Length(1), // AC
                Constraint::Length(1), // PSP
                Constraint::Length(1), // RAD
                Constraint::Length(1),
            ])
            .split(right[3].inner(Margin::new(1, 0)));
        // EL
        let el = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::ElectricLoad)
            .unwrap();
        frame.render_widget(
            Paragraph::new(format!(
                "EL: {}",
                if el.value == 0.0 { "OFF" } else { "ON" }
            )),
            flags_layout[1],
        );
        // AC
        let ac = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::AcSwitch)
            .unwrap();
        frame.render_widget(
            Paragraph::new(format!(
                "AC: {}",
                if ac.value == 0.0 { "OFF" } else { "ON" }
            )),
            flags_layout[2],
        );
        // PSP
        let psp = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::PspSwitch)
            .unwrap();
        frame.render_widget(
            Paragraph::new(format!(
                "PSP: {}",
                if psp.value == 0.0 { "OFF" } else { "ON" }
            )),
            flags_layout[3],
        );
        // FAN
        let fan = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::RadiatorFan)
            .unwrap();
        frame.render_widget(
            Paragraph::new(format!(
                "FAN: {}",
                if fan.value == 0.0 { "OFF" } else { "ON" }
            )),
            flags_layout[4],
        );

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
        if poll(Duration::from_millis(100))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
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
