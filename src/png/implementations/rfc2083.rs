// rfc2083
// png version 1.0
//

const MAX_DATA_LEN: u32 = u32::MAX;

#[derive(Debug)]
pub enum PNGError {
    InvalidSignature,
    ChunkTypeMismatch(String),
}

use std::vec::IntoIter;

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
}

// png header chunk
#[derive(Debug)]
pub struct Chunk {
    len: [u8; 4],
    type_: [u8; 4],
    data: Vec<u8>,
    crc: [u8; 4],
}

impl Chunk {
    pub fn from_iter(iter: &mut IntoIter<u8>, count: &mut usize) -> Self {
        let len: [u8; 4] = (0..4)
            .map(|_| iter.next().unwrap())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();

        let type_ = (0..4)
            .map(|_| iter.next().unwrap())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();

        let datalen = size_from_arr(len);

        let data = (0..datalen)
            .map(|_| iter.next().unwrap())
            .collect::<Vec<u8>>();

        let crc = (0..4)
            .map(|_| iter.next().unwrap())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();

        *count -= (4 * 3) + datalen as usize;

        Self {
            len,
            type_,
            data,
            crc,
        }
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn type_name(&self) -> String {
        self.ty().into_iter().map(|b| b as char).collect::<String>()
    }
}

fn size_from_arr(arr: [u8; 4]) -> u32 {
    arr.into_iter().fold(0u32, |mut mask, b| {
        mask <<= 8;
        mask |= b as u32;
        mask
    })
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "IHDR {{\n   len: {},\n   type: {},\n   data: {:?},\n   crc: {:?}\n}}",
                size_from_arr(self.len),
                self.type_
                    .into_iter()
                    .map(|b| b as char)
                    .collect::<String>(),
                self.data,
                self.crc
            )
        )
    }
}

pub trait ProcessData<T>
where
    Self: PNGChunk,
    T: ChunkData,
{
    fn type_match(&self) -> bool {
        T::ty_lit() == self.ty_lit()
    }

    fn process(&self) -> Result<T, PNGError>;
}

impl ProcessData<IHDRData> for Chunk {
    fn type_match(&self) -> bool {
        IHDRData::ty_lit() == &self.ty_lit()
    }

    fn process(&self) -> Result<IHDRData, PNGError> {
        if !self.type_match() {
            return Err(PNGError::ChunkTypeMismatch(format!(
                "trait ProcessData is not implemeted for chunk type {} with return type {}Data",
                self.ty_lit(),
                "IHDR"
            )));
        }
        let mut data = self.data().into_iter();

        Ok(IHDRData {
            width: stream_octets_and_u32(&mut data),
            height: stream_octets_and_u32(&mut data),
            bit_depth: *data.next().unwrap(),
            color_type: *data.next().unwrap(),
            compression_method: *data.next().unwrap(),
            filter_method: *data.next().unwrap(),
            interlace_method: *data.next().unwrap(),
        })
    }
}

pub trait PNGChunk {
    fn ty_lit(&self) -> String;
}

impl PNGChunk for Chunk {
    fn ty_lit(&self) -> String {
        self.type_
            .into_iter()
            .map(|b| b as char)
            .collect::<String>()
    }
}

pub trait ChunkData {
    fn ty_lit() -> &'static str;
}

impl ChunkData for IHDRData {
    fn ty_lit() -> &'static str {
        "IHDR"
    }
}

#[derive(Debug)]
pub struct IHDRData {
    width: u32,
    height: u32,
    // NOTE it's annoying how all these are just 1 bit long
    // but have to be stored in a single octet each
    // sono_hoka: u8,
    bit_depth: u8,
    color_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

// impl IHDRData {
//     pub fn from_data(data: &[u8]) -> Self {
//         let mut data = data.into_iter();
//
//         Self {
//             width: stream_octets_and_u32(&mut data),
//             height: stream_octets_and_u32(&mut data),
//             bit_depth: *data.next().unwrap(),
//             color_type: *data.next().unwrap(),
//             compression_method: *data.next().unwrap(),
//             filter_method: *data.next().unwrap(),
//             interlace_method: *data.next().unwrap(),
//         }
//     }
// }

fn stream_octets_and_u32(octets: &mut std::slice::Iter<u8>) -> u32 {
    (0..4)
        .into_iter()
        .map(|_| octets.next().unwrap())
        .fold(0u32, |mut mask, b| {
            mask <<= 8;
            mask |= *b as u32;

            mask
        })
}

impl Chunk {
    fn ty(&self) -> [u8; 4] {
        self.type_
    }

    fn is_critical(&self) -> bool {
        self.ty()[0] & 16 == 0
    }

    fn is_ancilary(&self) -> bool {
        self.ty()[0] & 16 == 16
    }

    fn is_public(&self) -> bool {
        self.ty()[1] & 16 == 0
    }

    fn is_private(&self) -> bool {
        self.ty()[1] & 16 == 16
    }

    fn is_version_1_0(&self) -> bool {
        self.ty()[2] & 16 == 0
    }

    fn is_safe_to_copy(&self) -> bool {
        self.ty()[3] & 16 == 16
    }
    fn is_unsafe_to_copy(&self) -> bool {
        self.ty()[3] & 16 == 0
    }
}

// TODO no need for a trait
// all chunks follow the same storage strategy
// then first chunkify the raw data

// DOCS PLTE chunk
// WARN when color type == 3 the PLTE chunk must appear
// WARN when color type == 2 || 6 PLTE chunk could appaer
// WARN when color type == 0 || 4 PLTE chunk must not appear
// NOTE must precede the 1st IDAT chunk
// NOTE there must be no more than one PLTE chunk
