// rfc2083
// png version 1.0
//

const MAX_DATA_LEN: u32 = u32::MAX;

pub fn dump_raw(value: impl AsRef<str> + Into<String>) -> Vec<u8> {
    std::fs::read(value.as_ref()).unwrap()
}

#[derive(Debug)]
pub enum PNGError {
    InvalidSignature,
}

use std::vec::IntoIter;

pub fn validate_signature(bytes: Vec<u8>) -> Result<IntoIter<u8>, PNGError> {
    let mut bytes = bytes.into_iter();
    if <Vec<u8> as TryInto<[u8; 8]>>::try_into(
        (0..8)
            .map(|_| bytes.next().unwrap_or(0))
            .collect::<Vec<u8>>(),
    )
    .unwrap()
        != [137, 80, 78, 71, 13, 10, 26, 10]
    {
        return Err(PNGError::InvalidSignature);
    }

    Ok(bytes)
}

// png header chunk
#[derive(Debug)]
pub struct IHDR {
    len: [u8; 4],
    type_: [u8; 4],
    data: Vec<u8>,
    crc: [u8; 4],
}

impl IHDR {
    pub fn from_iter(iter: &mut IntoIter<u8>) -> Self {
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

        let data = (0..size_from_arr(len))
            .map(|_| iter.next().unwrap())
            .collect::<Vec<u8>>();

        let crc = (0..4)
            .map(|_| iter.next().unwrap())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();

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
}

fn size_from_arr(arr: [u8; 4]) -> u32 {
    arr.into_iter().fold(0u32, |mut mask, b| {
        mask <<= 8;
        mask |= b as u32;
        mask
    })
}

impl std::fmt::Display for IHDR {
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

impl IHDRData {
    pub fn from_data(data: &[u8]) -> Self {
        let mut data = data.into_iter();

        Self {
            width: stream_octets_and_u32(&mut data),
            height: stream_octets_and_u32(&mut data),
            bit_depth: *data.next().unwrap(),
            color_type: *data.next().unwrap(),
            compression_method: *data.next().unwrap(),
            filter_method: *data.next().unwrap(),
            interlace_method: *data.next().unwrap(),
        }
    }
}

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

pub struct IDAT {}

pub struct IEND {}

trait ChunkProps {
    fn ty(&self) -> [u8; 4];

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

impl ChunkProps for IHDR {
    fn ty(&self) -> [u8; 4] {
        self.type_
    }
}
