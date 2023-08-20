mod app;
mod args;
mod cartain_codec;
mod gps;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{stdout, Stdout};

use crate::app::App;
use crate::args::Args;
use crate::gps::Gps;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut terminal = setup_terminal()?;

    let gps = Gps::new(args.device().to_string());
    let mut app = App::new();
    let res = app.run(&mut terminal, gps.rx);

    restore_terminal(&mut terminal)?;

    res
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);

    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    Ok(())
}
