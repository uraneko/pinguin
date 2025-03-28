// rfc2083
// png version 1.0
use std::vec::IntoIter;

mod chunks;
mod data;

use chunks::Chunk;

pub use data::{IHDR, bkGD, cHRM, pHYs, tEXt};

const MAX_DATA_LEN: u32 = u32::MAX;

#[derive(Debug)]
pub enum PNGError {
    InvalidSignature,
    ChunkTypeMismatch(String),
}

#[derive(Debug)]
pub struct RawPNG {
    state: IntoIter<u8>,
    len: usize,
}

impl RawPNG {
    pub fn from_file_name(file: &str) -> Result<Self, PNGError> {
        let state = std::fs::read(file).unwrap();
        let mut len = state.len();

        let mut state = state.into_iter();
        if let Err(e) = RawPNG::validate_signature(&mut state) {
            return Err(e);
        }
        len = len - 8;

        Ok(Self { state, len })
    }

    fn validate_signature(iter: &mut IntoIter<u8>) -> Result<(), PNGError> {
        if <Vec<u8> as TryInto<[u8; 8]>>::try_into(
            (0..8)
                .map(|_| iter.next().unwrap_or(0))
                .collect::<Vec<u8>>(),
        )
        .unwrap()
            != [137, 80, 78, 71, 13, 10, 26, 10]
        {
            return Err(PNGError::InvalidSignature);
        }

        Ok(())
    }

    fn next(&mut self) -> Option<u8> {
        self.state.next()
    }

    fn into_iter(self) -> IntoIter<u8> {
        self.state
    }

    fn len(&self) -> usize {
        self.len
    }
}

pub struct PNGChunks {
    state: Vec<Chunk>,
}

impl PNGChunks {
    pub fn from_raw(raw: RawPNG) -> Self {
        let mut len = raw.len();
        let mut iter = raw.into_iter();
        // WARN brittle error handling
        let mut chunks = vec![];
        while len > 0 {
            chunks.push(Chunk::from_iter(&mut iter, &mut len));
        }

        Self { state: chunks }
    }

    pub fn len(&self) -> usize {
        self.state.len()
    }

    pub fn chunks(&self) -> &[Chunk] {
        self.state.as_slice()
    }

    pub fn pop(&mut self) -> Chunk {
        self.state.pop().unwrap()
    }

    pub fn names(&self) -> Vec<String> {
        self.chunks()
            .into_iter()
            .map(|chunk| chunk.type_name())
            .collect::<Vec<String>>()
    }

    pub fn datas(&self) -> Vec<String> {
        self.chunks()
            .into_iter()
            .map(|chunk| chunk.type_name() + ", " + &format!("{}", chunk.data().len()))
            .collect::<Vec<String>>()
    }
}
