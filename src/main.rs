mod args;
mod gps;

use anyhow::{anyhow, Result};
use clap::Parser;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::Print,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::stdout;

use crate::args::Args;
use crate::gps::{Gps, Message};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;

    execute!(stdout, MoveTo(0, 0), Print("GPSInfo for Cartain GPSDXO"))?;

    execute!(stdout, MoveTo(0, 2), Print("Current:"))?;
    execute!(stdout, MoveTo(0, 3), Print("Deviation current:"))?;
    execute!(stdout, MoveTo(0, 4), Print("Deviation accumulated:"))?;
    execute!(stdout, MoveTo(0, 5), Print("DAC1 value:"))?;
    execute!(stdout, MoveTo(0, 6), Print("DAC2 value:"))?;
    execute!(stdout, MoveTo(0, 7), Print("Deviation:"))?;

    let gps = Gps::new(args.device().to_string());

    let mut stopped = false;

    while !stopped {
        let message = gps.rx.try_recv()?;
        match process_message(&message) {
            Ok(_) => {}
            Err(_) => stopped = true,
        }
    }

    execute!(stdout, Show, LeaveAlternateScreen)?;

    Ok(())
}

pub fn process_message(message: &Message) -> Result<()> {
    let mut stdout = stdout();

    match message {
        Message::Curr(data) => {
            execute!(
                stdout,
                MoveTo(23, 2),
                Clear(ClearType::UntilNewLine),
                Print(format!("{}", data))
            )?;
        }
        Message::DevCurr(data) => {
            execute!(
                stdout,
                MoveTo(23, 3),
                Clear(ClearType::UntilNewLine),
                Print(format!("{} Hz", data))
            )?;
        }
        Message::DevAccum(data) => {
            execute!(
                stdout,
                MoveTo(23, 4),
                Clear(ClearType::UntilNewLine),
                Print(format!("{} Hz", data))
            )?;
        }
        Message::DAC1(data) => {
            execute!(
                stdout,
                MoveTo(23, 5),
                Clear(ClearType::UntilNewLine),
                Print(format!("{}", data))
            )?;
        }
        Message::DAC2(data) => {
            execute!(
                stdout,
                MoveTo(23, 6),
                Clear(ClearType::UntilNewLine),
                Print(format!("{}", data))
            )?;
        }
        Message::Deviation(data) => {
            execute!(
                stdout,
                MoveTo(23, 7),
                Clear(ClearType::UntilNewLine),
                Print(format!("{} ppb", data))
            )?;
        }
        Message::SerialError(e) => {
            return Err(anyhow!(e.clone()));
        }
    }

    Ok(())
}
