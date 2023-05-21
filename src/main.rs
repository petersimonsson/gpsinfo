mod args;

use anyhow::Result;
use clap::Parser;
use futures::stream::StreamExt;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::{Decoder, LinesCodec};

use crate::args::Args;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("Device: {}", args.device());

    let mut port = tokio_serial::new(args.device(), 115200).open_native_async()?;
    port.set_exclusive(false)?;

    let codec = LinesCodec::new();
    let mut reader = codec.framed(port);

    let mut stopped = false;
    let mut lines: Vec<String> = Vec::new();
    let mut last_line = false;

    while !stopped {
        tokio::select! {
            line = reader.next() => {
                if let Some(line) = line {
                    let line = line?;

                    if line.starts_with("*") {
                        last_line = true;
                    }

                    lines.push(line);
                } else {
                    stopped = true;
                }
            }
        }

        if last_line {
            last_line = false;

            if lines.len() == 5 {
                for line in &lines {
                    if let Some((_, line)) = line.split_once(' ') {
                        println!("{}", line);
                    }
                }
            }

            lines.clear();
        }
    }

    Ok(())
}
