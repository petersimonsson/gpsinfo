use clap::Parser;

#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Args {
    device: String,
}

impl Args {
    pub fn device(&self) -> &str {
        &self.device
    }
}
