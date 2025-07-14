use clap::Parser;
use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};
use suzui_rs::{
    sdl::SuzukiSdlViewer,
    widgets::{
        airflow::AirflowBlock, electrical::ElectricalBlock, engine::EngineSpeedBlock,
        flags::FlagsBlock, fuel_ignition::FuelIgnitionBlock, temperature::TemperatureBlock,
        throttle::ThrottleBlock, vehicle::VehicleBlock,
    },
};

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
            self.handle_crossterm_events(should_simulate)?;
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
        // brand new shrand new
        let engine_speed_block = EngineSpeedBlock::new(&self.sdl_viewer.engine_context);
        let airflow_block = AirflowBlock::new(&self.sdl_viewer.engine_context);
        let fuel_ignition_block = FuelIgnitionBlock::new(&self.sdl_viewer.engine_context);
        let vehicle_block = VehicleBlock::new(&self.sdl_viewer.engine_context);
        let throttle_block = ThrottleBlock::new(&self.sdl_viewer.engine_context);
        let temperature_block = TemperatureBlock::new(&self.sdl_viewer.engine_context);
        let electrical_block = ElectricalBlock::new(&self.sdl_viewer.engine_context);
        let flags_block = FlagsBlock::new(&self.sdl_viewer.engine_context);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());
        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(9), // engine
                Constraint::Length(6), // fuel/ignition
                Constraint::Length(6), // temperatures
            ])
            .split(layout[0]);
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(7), // throttle
                Constraint::Length(4), // load
                Constraint::Length(3), // electrical
                Constraint::Length(3), // vehicle
                Constraint::Length(4), // flags
            ])
            .split(layout[1]);

        frame.render_widget(engine_speed_block, left[0]);
        frame.render_widget(fuel_ignition_block, left[1]);
        frame.render_widget(temperature_block, left[2]);
        frame.render_widget(throttle_block, right[0]);
        frame.render_widget(airflow_block, right[1]);
        frame.render_widget(electrical_block, right[2]);
        frame.render_widget(vehicle_block, right[3]);
        frame.render_widget(flags_block, right[4]);

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
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self, should_simulate: bool) -> Result<()> {
        if poll(Duration::from_millis(if should_simulate { 100 } else { 0 }))? {
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
