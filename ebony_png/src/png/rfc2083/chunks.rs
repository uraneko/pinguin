use super::PNGError;
use std::vec::IntoIter;

// png header chunk
#[derive(Debug)]
pub struct Chunk {
    len: [u8; 4],
    type_: [u8; 4],
    data: Vec<u8>,
    crc: [u8; 4],
}

impl Chunk {
    pub fn sample(len: [u8; 4], type_: [u8; 4], data: Vec<u8>, crc: [u8; 4]) -> Self {
        Self {
            len,
            type_,
            data,
            crc,
        }
    }

    pub fn len(&self) -> usize {
        self.len.into_iter().fold(0u32, |mut mask, b| {
            mask <<= 8;
            mask |= b as u32;
            mask
        }) as usize
    }
}

impl Chunk {
    // returns a new chunk from an iterator of bytes (octets)
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

    // returns an immutable ref to the data of the chunk
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    // returns the chunk type as a human readable string
    pub fn type_name(&self) -> String {
        self.ty().into_iter().map(|b| b as char).collect::<String>()
    }

    pub fn crc(&self) -> [u8; 4] {
        self.crc
    }
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

// DOCS PLTE chunk
// WARN when color type == 3 the PLTE chunk must appear
// WARN when color type == 2 || 6 PLTE chunk could appaer
// WARN when color type == 0 || 4 PLTE chunk must not appear
// NOTE must precede the 1st IDAT chunk
// NOTE there must be no more than one PLTE chunk
