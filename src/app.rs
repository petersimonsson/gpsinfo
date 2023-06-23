use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::Local;
use crossterm::event::{self, Event, KeyCode};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    widgets::{Axis, Block, Borders, Cell, Chart, Dataset, GraphType, Row, Table},
    Frame, Terminal,
};

use crate::gps::{Gps, Message};

pub struct App {
    gps: Gps,
    current: Vec<(f64, f64)>,
    devcurr: Vec<(f64, f64)>,
    devaccum: Vec<(f64, f64)>,
    dac1: Option<u32>,
    dac2: Option<u32>,
    deviation: Vec<(f64, f64)>,
}

impl App {
    pub fn new(device: &str) -> Self {
        App {
            gps: Gps::new(device.to_string()),
            current: Vec::new(),
            devcurr: Vec::new(),
            devaccum: Vec::new(),
            dac1: None,
            dac2: None,
            deviation: Vec::new(),
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
            .constraints(
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                ]
                .as_ref(),
            )
            .split(f.size());

        let list = self.generate_data_list();
        let table = Table::new(list)
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
            .block(Block::default().title("GPSDXO Data").borders(Borders::ALL));

        f.render_widget(table, chunks[0]);

        let now = Local::now().timestamp() as f64;
        let beginning = now - 300.0;

        let datasets = vec![Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&self.current)];

        let chart = Chart::new(datasets)
            .block(Block::default().title("Current").borders(Borders::ALL))
            .x_axis(Axis::default().bounds([beginning, now]))
            .y_axis(Axis::default().bounds([79999995.0, 80000005.0]));

        f.render_widget(chart, chunks[1]);

        let datasets = vec![
            Dataset::default()
                .name("Current")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.devcurr),
            Dataset::default()
                .name("Accumulated")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Red))
                .data(&self.devaccum),
        ];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title("Deviation(Hz)")
                    .borders(Borders::ALL),
            )
            .x_axis(Axis::default().bounds([beginning, now]))
            .y_axis(Axis::default().bounds([-1.0, 2.0]));

        f.render_widget(chart, chunks[2]);

        let datasets = vec![Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&self.deviation)];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title("Deviation(ppb)")
                    .borders(Borders::ALL),
            )
            .x_axis(Axis::default().bounds([beginning, now]))
            .y_axis(Axis::default().bounds([0.0, 2.0]));

        f.render_widget(chart, chunks[3]);
    }

    fn process_message(&mut self, message: &Message) -> Result<()> {
        match message {
            Message::Curr(data) => {
                self.current
                    .push((Local::now().timestamp() as f64, *data as f64));

                if self.current.len() > 300 {
                    self.current.remove(0);
                }
            }
            Message::DevCurr(data) => {
                self.devcurr.push((Local::now().timestamp() as f64, *data));

                if self.devcurr.len() > 300 {
                    self.devcurr.remove(0);
                }
            }
            Message::DevAccum(data) => {
                self.devaccum.push((Local::now().timestamp() as f64, *data));

                if self.devaccum.len() > 300 {
                    self.devaccum.remove(0);
                }
            }
            Message::DAC1(data) => {
                self.dac1 = Some(*data);
            }
            Message::DAC2(data) => {
                self.dac2 = Some(*data);
            }
            Message::Deviation(data) => {
                self.deviation
                    .push((Local::now().timestamp() as f64, *data));

                if self.deviation.len() > 300 {
                    self.deviation.remove(0);
                }
            }
            Message::SerialError(e) => {
                return Err(anyhow!(e.clone()));
            }
        }

        Ok(())
    }

    fn generate_data_list(&self) -> Vec<Row> {
        let current = match self.current.last() {
            Some(data) => data.1.to_string(),
            None => "".to_string(),
        };
        let devcurr = match self.devcurr.last() {
            Some(data) => data.1.to_string(),
            None => "".to_string(),
        };
        let devaccum = match self.devaccum.last() {
            Some(data) => data.1.to_string(),
            None => "".to_string(),
        };
        let dac1 = match self.dac1 {
            Some(dac1) => dac1.to_string(),
            None => "".to_string(),
        };
        let dac2 = match self.dac2 {
            Some(dac2) => dac2.to_string(),
            None => "".to_string(),
        };
        let deviation = match self.deviation.last() {
            Some(data) => data.1.to_string(),
            None => "".to_string(),
        };

        vec![
            Row::new(vec![Cell::from("Current"), Cell::from(current)]),
            Row::new(vec![
                Cell::from("Deviation current"),
                Cell::from(format!("{}Hz", devcurr)),
            ]),
            Row::new(vec![
                Cell::from("Deviation accumulated"),
                Cell::from(format!("{}Hz", devaccum)),
            ]),
            Row::new(vec![Cell::from("DAC1"), Cell::from(dac1)]),
            Row::new(vec![Cell::from("DAC2"), Cell::from(dac2)]),
            Row::new(vec![
                Cell::from("Deviation"),
                Cell::from(format!("{}ppb", deviation)),
            ]),
        ]
    }
}
