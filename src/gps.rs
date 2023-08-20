use std::sync::Arc;

use crossbeam::channel::{self, Receiver, Sender};
use futures::StreamExt;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;

use crate::cartain_codec::{CartainCodec, CartainMessage};

pub enum Message {
    Data(CartainMessage),
    SerialError(Arc<str>),
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
            let mut port = match tokio_serial::new(device, 115200).open_native_async() {
                Ok(port) => port,
                Err(e) => {
                    line_tx
                        .send(Message::SerialError(e.to_string().into()))
                        .unwrap();
                    return;
                }
            };
            if let Err(e) = port.set_exclusive(false) {
                line_tx
                    .send(Message::SerialError(e.to_string().into()))
                    .unwrap();
            }

            let codec = CartainCodec::new();
            let mut reader = codec.framed(port);

            loop {
                match reader.next().await {
                    Some(Ok(message)) => {
                        line_tx.send(Message::Data(message)).unwrap();
                    }
                    Some(Err(e)) => line_tx
                        .send(Message::SerialError(e.to_string().into()))
                        .unwrap(),
                    None => {}
                }
            }
        });

        Gps { rx, _tx: tx }
    }
}
