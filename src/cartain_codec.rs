use bytes::BytesMut;
use tokio_util::codec::Decoder;

pub enum CartainMessage {
    Curr(u64),
    DevCurr(f64),
    DevAccum(f64),
    Deviation(f64),
    DAC1(u32),
    DAC2(u32),
}

impl CartainMessage {
    fn from_str(line: &str) -> Option<Self> {
        if line.starts_with("# Curr:") {
            let (_, data) = line.rsplit_once(' ').unwrap();

            if let Ok(data) = data.parse() {
                return Some(CartainMessage::Curr(data));
            }
        } else if line.starts_with("# Deviation current:") {
            let (_, data) = line.rsplit_once(' ').unwrap();
            let data = data.strip_suffix("Hz").unwrap();

            if let Ok(data) = data.parse() {
                return Some(CartainMessage::DevCurr(data));
            }
        } else if line.starts_with("# Deviation accum:") {
            let (_, data) = line.rsplit_once(' ').unwrap();
            let data = data.strip_suffix("Hz").unwrap();

            if let Ok(data) = data.parse() {
                return Some(CartainMessage::DevAccum(data));
            }
        } else if line.starts_with("# New DAC1 value") {
            let (_, data) = line.rsplit_once(' ').unwrap();

            if let Ok(data) = data.parse() {
                return Some(CartainMessage::DAC1(data));
            }
        } else if line.starts_with("# New DAC2 value") {
            let (_, data) = line.rsplit_once(' ').unwrap();

            if let Ok(data) = data.parse() {
                return Some(CartainMessage::DAC2(data));
            }
        } else if line.starts_with('*') {
            let (_, data) = line.split_once(' ').unwrap();
            let data = data.strip_suffix(" ppb").unwrap();

            if let Ok(data) = data.parse() {
                return Some(CartainMessage::Deviation(data));
            }
        }

        None
    }
}

pub struct CartainCodec {
    start_index: usize,
}

impl CartainCodec {
    pub fn new() -> Self {
        CartainCodec { start_index: 0 }
    }
}

impl Decoder for CartainCodec {
    type Item = CartainMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline_offset = src[self.start_index..].iter().position(|b| *b == b'\n');

        match newline_offset {
            Some(offset) => {
                let index = self.start_index + offset;
                self.start_index = 0;
                let line = src.split_to(index + 1);
                let line = &line[..line.len() - 1];
                let line = remove_carriage_return(line);
                let line = to_str(line)?;
                Ok(CartainMessage::from_str(line))
            }
            None => {
                self.start_index = src.len();
                Ok(None)
            }
        }
    }
}

fn remove_carriage_return(s: &[u8]) -> &[u8] {
    if let Some(&b'\r') = s.last() {
        &s[..s.len() - 1]
    } else {
        s
    }
}

fn to_str(src: &[u8]) -> Result<&str, std::io::Error> {
    std::str::from_utf8(src).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to decode input as UTF8",
        )
    })
}
