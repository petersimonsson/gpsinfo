use crossbeam::channel::{self, Receiver, Sender};
use futures::StreamExt;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::{Decoder, LinesCodec};

pub enum Message {
    Curr(u64),
    DevCurr(f64),
    DevAccum(f64),
    Deviation(f32),
    DAC1(u32),
    DAC2(u32),
    Error,
}

pub struct Gps {
    pub rx: Receiver<Message>,
    _tx: Sender<Message>,
}

impl Gps {
    pub fn new(device: String) -> Self {
        let (tx, rx) = channel::unbounded();
        let line_tx = tx.clone();

        tokio::spawn(async move {
            let mut port = tokio_serial::new(device, 115200)
                .open_native_async()
                .unwrap(); // TODO: Less unwrap!
            port.set_exclusive(false).unwrap(); // TODO: Less unwrap!

            let codec = LinesCodec::new();
            let mut reader = codec.framed(port);

            loop {
                if let Some(line) = reader.next().await {
                    let line = line.unwrap(); // TODO: Less unwrap!

                    if line.starts_with("# Curr:") {
                        let (_, data) = line.rsplit_once(' ').unwrap();
                        line_tx.send(Message::Curr(data.parse().unwrap())).unwrap();
                    } else if line.starts_with("# Deviation current:") {
                        let (_, data) = line.rsplit_once(' ').unwrap();
                        let data = data.strip_suffix("Hz").unwrap();
                        line_tx
                            .send(Message::DevCurr(data.parse().unwrap()))
                            .unwrap();
                    } else if line.starts_with("# Deviation accum:") {
                        let (_, data) = line.rsplit_once(' ').unwrap();
                        let data = data.strip_suffix("Hz").unwrap();
                        line_tx
                            .send(Message::DevAccum(data.parse().unwrap()))
                            .unwrap();
                    } else if line.starts_with("# New DAC1 value") {
                        let (_, data) = line.rsplit_once(' ').unwrap();
                        line_tx.send(Message::DAC1(data.parse().unwrap())).unwrap();
                    } else if line.starts_with("# New DAC2 value") {
                        let (_, data) = line.rsplit_once(' ').unwrap();
                        line_tx.send(Message::DAC2(data.parse().unwrap())).unwrap();
                    } else if line.starts_with("*") {
                        let (_, data) = line.split_once(' ').unwrap();
                        let data = data.strip_suffix(" ppb").unwrap();
                        line_tx
                            .send(Message::Deviation(data.parse().unwrap()))
                            .unwrap();
                    }
                } else {
                    line_tx.send(Message::Error).unwrap(); // TODO: Less unwrap!
                }
            }
        });

        Gps { rx, _tx: tx }
    }
}
