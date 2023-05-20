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

    while let Some(line_result) = reader.next().await {
        let line = line_result?;
        println!("{}", line);
    }

    Ok(())
}
