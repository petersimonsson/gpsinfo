use std::{io::Stdout, time::Duration};

use anyhow::{anyhow, Result};
use chrono::Local;
use crossbeam::channel::Receiver;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols,
    widgets::{Axis, Block, Borders, Cell, Chart, Dataset, GraphType, Row, Table},
    Frame, Terminal,
};

use crate::{cartain_codec::CartainMessage, gps::Message};

pub struct App {
    current: Vec<(f64, f64)>,
    devcurr: Vec<(f64, f64)>,
    devaccum: Vec<(f64, f64)>,
    dac1: Option<u32>,
    dac2: Option<u32>,
    deviation: Vec<(f64, f64)>,
}

impl App {
    pub fn new() -> Self {
        App {
            current: Vec::new(),
            devcurr: Vec::new(),
            devaccum: Vec::new(),
            dac1: None,
            dac2: None,
            deviation: Vec::new(),
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        data_channel: Receiver<Message>,
    ) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Ok(true) = event::poll(Duration::from_millis(250)) {
                if let Event::Key(key) = event::read()? {
                    if let KeyCode::Char('q') = key.code {
                        return Ok(());
                    }
                }
            }

            for message in data_channel.try_iter() {
                self.process_message(&message)?;
            }
        }
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(8),
                    Constraint::Percentage(20),
                    Constraint::Percentage(35),
                    Constraint::Percentage(35),
                ]
                .as_ref(),
            )
            .split(f.area());

        let list = self.generate_data_list();
        let table = Table::new(list, &[Constraint::Length(21), Constraint::Length(15)])
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
            Message::Data(message) => match message {
                CartainMessage::Curr(data) => {
                    self.current
                        .push((Local::now().timestamp() as f64, *data as f64));

                    if self.current.len() > 300 {
                        self.current.remove(0);
                    }
                }
                CartainMessage::DevCurr(data) => {
                    self.devcurr.push((Local::now().timestamp() as f64, *data));

                    if self.devcurr.len() > 300 {
                        self.devcurr.remove(0);
                    }
                }
                CartainMessage::DevAccum(data) => {
                    self.devaccum.push((Local::now().timestamp() as f64, *data));

                    if self.devaccum.len() > 300 {
                        self.devaccum.remove(0);
                    }
                }
                CartainMessage::DAC1(data) => {
                    self.dac1 = Some(*data);
                }
                CartainMessage::DAC2(data) => {
                    self.dac2 = Some(*data);
                }
                CartainMessage::Deviation(data) => {
                    self.deviation
                        .push((Local::now().timestamp() as f64, *data));

                    if self.deviation.len() > 300 {
                        self.deviation.remove(0);
                    }
                }
            },
            Message::SerialError(e) => {
                return Err(anyhow!(e.clone()));
            }
        }

        Ok(())
    }

    fn generate_data_list(&self) -> Vec<Row<'_>> {
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
            Row::new(vec![Cell::from("Current".bold()), Cell::from(current)]),
            Row::new(vec![
                Cell::from("Deviation current".bold()),
                Cell::from(format!("{}Hz", devcurr)),
            ]),
            Row::new(vec![
                Cell::from("Deviation accumulated".bold()),
                Cell::from(format!("{}Hz", devaccum)),
            ]),
            Row::new(vec![Cell::from("DAC1".bold()), Cell::from(dac1)]),
            Row::new(vec![Cell::from("DAC2".bold()), Cell::from(dac2)]),
            Row::new(vec![
                Cell::from("Deviation".bold()),
                Cell::from(format!("{}ppb", deviation)),
            ]),
        ]
    }
}
