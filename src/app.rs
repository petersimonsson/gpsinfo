use std::time::Duration;

use anyhow::{anyhow, Result};
use crossterm::event::{self, Event, KeyCode};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame, Terminal,
};

use crate::gps::{Gps, Message};

pub struct App {
    gps: Gps,
    current: u64,
    devcurr: f64,
    devaccum: f64,
    dac1: u32,
    dac2: u32,
    deviation: f32,
}

impl App {
    pub fn new(device: &str) -> Self {
        App {
            gps: Gps::new(device.to_string()),
            current: 0,
            devcurr: 0.0,
            devaccum: 0.0,
            dac1: 0,
            dac2: 0,
            deviation: 0.0,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Ok(true) = event::poll(Duration::from_secs(0)) {
                if let Event::Key(key) = event::read()? {
                    if let KeyCode::Char('q') = key.code {
                        return Ok(());
                    }
                }
            }
            if let Ok(message) = self.gps.rx.try_recv() {
                self.process_message(&message)?;
            }
        }
    }

    fn ui<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(f.size());

        let list = self.generate_data_list();
        let table = Table::new(list)
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
            .block(Block::default().title("GPSDXO Data").borders(Borders::ALL));

        f.render_widget(table, chunks[0]);
    }

    fn process_message(&mut self, message: &Message) -> Result<()> {
        match message {
            Message::Curr(data) => {
                self.current = *data;
            }
            Message::DevCurr(data) => {
                self.devcurr = *data;
            }
            Message::DevAccum(data) => {
                self.devaccum = *data;
            }
            Message::DAC1(data) => {
                self.dac1 = *data;
            }
            Message::DAC2(data) => {
                self.dac2 = *data;
            }
            Message::Deviation(data) => {
                self.deviation = *data;
            }
            Message::SerialError(e) => {
                return Err(anyhow!(e.clone()));
            }
        }

        Ok(())
    }

    fn generate_data_list(&self) -> Vec<Row> {
        vec![
            Row::new(vec![
                Cell::from("Current"),
                Cell::from(self.current.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Deviation current"),
                Cell::from(format!("{}Hz", self.devcurr)),
            ]),
            Row::new(vec![
                Cell::from("Deviation accumulated"),
                Cell::from(format!("{}Hz", self.devaccum)),
            ]),
            Row::new(vec![Cell::from("DAC1"), Cell::from(self.dac1.to_string())]),
            Row::new(vec![Cell::from("DAC2"), Cell::from(self.dac2.to_string())]),
            Row::new(vec![
                Cell::from("Deviation"),
                Cell::from(format!("{}ppb", self.deviation)),
            ]),
        ]
    }
}
