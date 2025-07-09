use clap::Parser;
use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use suzui_rs::sdl::{ScanToolParameter, SuzukiSdlViewer};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    simulate: bool,
}

fn main() -> color_eyre::Result<()> {
    let args = Args::parse();
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal, args.simulate);
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
    pub fn run(mut self, mut terminal: DefaultTerminal, should_simulate: bool) -> Result<()> {
        self.running = true;
        if !should_simulate {
            self.sdl_viewer.connect();
        }
        while self.running {
            self.sdl_viewer.update_raw_data(should_simulate);
            self.sdl_viewer.update_processed_data();
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn render_engine_speed(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::new().borders(Borders::ALL).title(Span::styled(
                "ENGINE SPEED",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
            area,
        );
        let engine_speed_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), // header
                Constraint::Length(5), // rpm
                Constraint::Length(1), // desired idle
                Constraint::Length(3), // isc flow duty
                Constraint::Length(1), // footer
            ])
            .split(area.inner(Margin::new(1, 0)));
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
        let isc_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(engine_speed_layout[3]);
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
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ));
        frame.render_widget(rpm_gauge, engine_speed_layout[1]);
        frame.render_widget(
            Paragraph::new(format!("Target: {} RPM", desired_idle.value)),
            engine_speed_layout[2],
        );
        let isc_gauge = Gauge::default()
            .percent(isc_flow_duty.value as u16)
            .label(format!("{} %", isc_flow_duty.value));
        frame.render_widget(Text::raw("ISC:"), isc_row[0].inner(Margin::new(0, 1)));
        frame.render_widget(isc_gauge, isc_row[1]);
    }

    fn render_throttle_block(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::new().borders(Borders::ALL).title(Span::styled(
                "THROTTLE",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
            area,
        );
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
            .split(area.inner(Margin::new(1, 0)));
        let throttle_value = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::AbsoluteThrottlePosition)
            .unwrap();
        let tps_angle = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::ThrottleAngle)
            .unwrap();
        let ctp = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::ClosedThrottlePos)
            .unwrap();
        let throttle_gauge = Gauge::default()
            .percent(throttle_value.value as u16)
            .label(Span::styled(
                format!("{} °", tps_angle.value),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .gauge_style(if ctp.value == 1.0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            });
        frame.render_widget(throttle_gauge, throttle_block_layout[1]);
    }

    fn render_fuel_ignition_block(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::new().borders(Borders::ALL).title(Span::styled(
                "FUEL/IGNITION",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
            area,
        );
        let fuel_ignition_block_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Max(3),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area.inner(Margin::new(1, 0)));
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
        if fuel_cut.value == 0.0 {
            let inj_pw_percentage =
                ((inj_pw_current.value / 50.0 as f32) * 100.0).min(100.0) as u16;
            let inj_pw_gauge = Gauge::default()
                .percent(inj_pw_percentage)
                //.gauge_style(Style::default().fg(engine_rpm_color))
                .label(Span::styled(
                    format!("{:.1} ms", inj_pw_current.value),
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ));
            frame.render_widget(inj_pw_gauge, fuel_ignition_block_layout[1]);
        } else {
            let fuel_cut_gauge = Gauge::default()
                .percent(100)
                .gauge_style(Style::default().fg(Color::Green))
                .label(Span::styled(
                    "FUEL CUT",
                    Style::default().fg(Color::White).bg(Color::Black),
                ));
            frame.render_widget(fuel_cut_gauge, fuel_ignition_block_layout[1]);
        }
        frame.render_widget(
            Span::styled(
                format!("IGN ADV: {}", ignition_advance.value),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            fuel_ignition_block_layout[2],
        );
    }

    fn render_temperatures_block(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::new().borders(Borders::ALL).title(Span::styled(
                "TEMPERATURES",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
            area,
        );
        let temperature_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Length(1),
            ])
            .split(area.inner(Margin::new(1, 0)));
        let coolant_temp_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
            .split(temperature_layout[1]);
        let intake_temp_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
            .split(temperature_layout[2]);
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
        frame.render_widget(
            Span::styled(
                "COOLANT:",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            coolant_temp_layout_text[1],
        );
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
            temp if temp < 101.0 => Color::Green,
            temp if temp < 105.0 => Color::Yellow,
            _ => Color::Red,
        };
        let rad_fan = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::RadiatorFan)
            .unwrap();
        let coolant_gauge = Gauge::default()
            .percent(coolant_percentage)
            .gauge_style(Style::default().fg(coolant_color))
            .label(Span::styled(
                if rad_fan.value == 0.0 {
                    format!("{} °C", coolant_current)
                } else {
                    format!("🌀 {} °C 🌀", coolant_current)
                },
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ));
        frame.render_widget(coolant_gauge, coolant_temp_layout[1]);
        frame.render_widget(
            Span::styled(
                "INTAKE:",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            intake_temp_layout_text[1],
        );
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
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ));
        frame.render_widget(intake_gauge, intake_temp_layout[1]);
    }

    fn render_airflow_block(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::new().borders(Borders::ALL).title(Span::styled(
                "AIRFLOW",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
            area,
        );
        let airflow_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area.inner(Margin::new(1, 0)));
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
            Span::styled(
                format!("MAP:  {:.2} kPa", map.value),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            airflow_layout[1],
        );
        frame.render_widget(
            Span::styled(
                format!("BARO: {:.2} kPa", baro.value),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            airflow_layout[2],
        );
    }

    fn render_electrical_block(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::new().borders(Borders::ALL).title(Span::styled(
                "ELECTRICAL",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
            area,
        );
        let electrical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Length(1)])
            .split(area.inner(Margin::new(1, 0)));
        let battery_voltage = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::BatteryVoltage)
            .unwrap();
        let engine_rpm = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::EngineSpeed)
            .unwrap();
        let battery_color = match engine_rpm.value {
            rpm if rpm > 0.0 => match battery_voltage.value {
                batt_volt if batt_volt < 13.2 => (Color::Red, Color::White),
                batt_volt if batt_volt < 15.2 => (Color::Black, Color::Green),
                _ => (Color::Red, Color::White),
            },
            _ => match battery_voltage.value {
                batt_volt if batt_volt < 12.2 => (Color::Red, Color::White),
                batt_volt if batt_volt < 12.4 => (Color::Black, Color::Yellow),
                batt_volt if batt_volt < 12.6 => (Color::Red, Color::Yellow),
                _ => (Color::Black, Color::White),
            },
        };
        frame.render_widget(
            Span::styled(
                format!("BATT: {:.1} V", battery_voltage.value),
                Style::default()
                    .bg(battery_color.0)
                    .fg(battery_color.1)
                    .add_modifier(Modifier::BOLD),
            ),
            electrical_layout[1],
        );
    }

    fn render_vehicle_block(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::new().borders(Borders::ALL).title(Span::styled(
                "VEHICLE",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
            area,
        );
        let speed_block = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Max(3)])
            .split(area.inner(Margin::new(1, 0)));
        let speed = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::VehicleSpeed)
            .unwrap();
        let speed_color = match speed.value {
            speed if speed < 60.0 => (Color::Black, Color::White),
            speed if speed <= 80.0 => (Color::Black, Color::Green),
            speed if speed <= 110.0 => (Color::Black, Color::Yellow),
            _ => (Color::Red, Color::White),
        };
        let speed = Span::styled(
            format!("Speed: {} km/h", speed.value),
            Style::default()
                .bg(speed_color.0)
                .fg(speed_color.1)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(speed, speed_block[1]);
    }

    fn render_flags_block(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::new().borders(Borders::ALL).title(Span::styled(
                "STATUS FLAGS",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
            area,
        );
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
            .split(area.inner(Margin::new(1, 0)));
        // EL
        let el = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::ElectricLoad)
            .unwrap();
        frame.render_widget(
            Span::styled(
                format!("EL: {}", if el.value == 0.0 { "OFF" } else { "ON" }),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            flags_layout[1],
        );
        // AC
        let ac = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::AcSwitch)
            .unwrap();
        frame.render_widget(
            Span::styled(
                format!("AC: {}", if ac.value == 0.0 { "OFF" } else { "ON" }),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            flags_layout[2],
        );
        // PSP
        let psp = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::PspSwitch)
            .unwrap();
        frame.render_widget(
            Span::styled(
                format!("PSP: {}", if psp.value == 0.0 { "OFF" } else { "ON" }),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            flags_layout[3],
        );
        // FAN
        let fan = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::RadiatorFan)
            .unwrap();
        frame.render_widget(
            Span::styled(
                format!("FAN: {}", if fan.value == 0.0 { "OFF" } else { "ON" }),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            flags_layout[4],
        );
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

        // brand new shrand new
        self.render_engine_speed(frame, left[0]);

        //self.render_fuel_ignition_block(frame, left[1]);
        let inj_pw = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::InjPulseWidthCyl1)
            .unwrap()
            .value;
        let fuel_cut = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::FuelCut)
            .unwrap()
            .value;
        let ignition_advance = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::IgnitionAdvance)
            .unwrap()
            .value;
        let fuel_ignition_block = FuelIgnitionBlock::new(
            inj_pw,
            ignition_advance as i8,
            if fuel_cut == 0.0 { true } else { false },
        );
        frame.render_widget(fuel_ignition_block, left[1]);

        let map = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::Map)
            .unwrap()
            .value;
        let baro = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::BarometricPressure)
            .unwrap()
            .value;
        let airflow_block = AirflowBlock::new(map, baro);
        frame.render_widget(airflow_block, left[2]);

        let speed_kmh: u8 = self
            .sdl_viewer
            .scan_tool_data
            .get(&ScanToolParameter::VehicleSpeed)
            .unwrap()
            .value as u8;
        let vehicle_block = VehicleBlock::new(speed_kmh);
        frame.render_widget(vehicle_block, left[3]);

        self.render_throttle_block(frame, right[0]);
        self.render_temperatures_block(frame, right[1]);
        self.render_electrical_block(frame, right[2]);
        self.render_flags_block(frame, right[3]);

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
            frame.render_widget(Paragraph::new(format!("{}", value.value)), area_value);
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

struct FuelIgnitionBlock {
    inj_pw: f32,
    fuel_cut: bool,
    ignition_advance: i8,
}

impl FuelIgnitionBlock {
    pub fn new(inj_pw: f32, ignition_advance: i8, fuel_cut: bool) -> Self {
        Self {
            inj_pw,
            fuel_cut,
            ignition_advance,
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
                Constraint::Length(1),
                Constraint::Max(3),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area.inner(Margin::new(1, 0)));
        if self.fuel_cut {
            let inj_pw_percentage = ((self.inj_pw / 50.0 as f32) * 100.0).min(100.0) as u16;
            Gauge::default()
                .percent(inj_pw_percentage)
                .label(Span::styled(
                    format!("{:.1} ms", self.inj_pw),
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ))
                .render(fuel_ignition_block_layout[1], buf);
        } else {
            Gauge::default()
                .percent(100)
                .gauge_style(Style::default().fg(Color::Green))
                .label(Span::styled(
                    "FUEL CUT",
                    Style::default().fg(Color::White).bg(Color::Black),
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

struct AirflowBlock {
    map: f32,
    baro: f32,
}

impl AirflowBlock {
    pub fn new(map: f32, baro: f32) -> Self {
        Self { map, baro }
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
                "AIRFLOW",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(area, buf);
        let airflow_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area.inner(Margin::new(1, 0)));
        Span::styled(
            format!("MAP:  {:.2} kPa", self.map),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .render(airflow_layout[1], buf);
        Span::styled(
            format!("BARO: {:.2} kPa", self.baro),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .render(airflow_layout[2], buf);
    }
}

struct VehicleBlock {
    speed: u8,
}

impl VehicleBlock {
    pub fn new(speed: u8) -> Self {
        Self { speed }
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
            .constraints(vec![Constraint::Length(1), Constraint::Max(3)])
            .split(area.inner(Margin::new(1, 0)));
        let speed_color = match self.speed {
            speed if speed < 60 => (Color::Black, Color::White),
            speed if speed <= 80 => (Color::Black, Color::Green),
            speed if speed <= 110 => (Color::Black, Color::Yellow),
            _ => (Color::Red, Color::White),
        };
        let speed = Span::styled(
            format!("Speed: {} km/h", self.speed),
            Style::default()
                .bg(speed_color.0)
                .fg(speed_color.1)
                .add_modifier(Modifier::BOLD),
        );
        speed.render(speed_block[1], buf);
    }
}
