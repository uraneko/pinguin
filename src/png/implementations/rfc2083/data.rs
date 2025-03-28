use super::PNGError;
use super::chunks::Chunk;
use std::slice::Iter;

use chunk_data::ChunkData;

#[derive(Debug, ChunkData)]
pub struct IHDR {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

struct ChunkProcess<'a> {
    color_type: Option<u8>,
    data: &'a [u8],
}

impl ChunkProcess<'_> {
    fn data(&self) -> &[u8] {
        self.data
    }

    fn ct(&self) -> u8 {
        self.color_type.unwrap()
    }
}

impl Chunk {
    fn process_with_ct<'a>(&'a self, ct: u8) -> ChunkProcess<'a> {
        ChunkProcess {
            color_type: Some(ct),
            data: self.data(),
        }
    }

    fn process<'a>(&'a self) -> ChunkProcess<'a> {
        ChunkProcess {
            color_type: None,
            data: self.data(),
        }
    }

    pub fn chunk_data<'a, T>(&'a self, ct: Option<u8>) -> Result<T, PNGError>
    where
        T: TryFrom<ChunkProcess<'a>, Error = PNGError>,
    {
        T::try_from(if let Some(ct) = ct {
            self.process_with_ct(ct)
        } else {
            self.process()
        })
    }
}

#[derive(Debug, ChunkData)]
pub struct cHRM {
    white_point_x: u32,
    white_point_y: u32,
    red_x: u32,
    red_y: u32,
    green_x: u32,
    green_y: u32,
    blue_x: u32,
    blue_y: u32,
}

const DIVISOR: f64 = 100_000.0;
const INCH: f64 = 0.0254; // in meters

impl cHRM {
    fn white_point(&self) -> [f64; 2] {
        [
            self.white_point_x as f64 / DIVISOR,
            self.white_point_y as f64 / DIVISOR,
        ]
    }

    fn red(&self) -> [f64; 2] {
        [self.red_x as f64 / DIVISOR, self.red_y as f64 / DIVISOR]
    }

    fn green(&self) -> [f64; 2] {
        [self.green_x as f64 / DIVISOR, self.green_y as f64 / DIVISOR]
    }

    fn blue(&self) -> [f64; 2] {
        [self.blue_x as f64 / DIVISOR, self.blue_y as f64 / DIVISOR]
    }
}

#[derive(Debug, ChunkData)]
pub enum bkGD {
    // add derive attribute :value from ct // would deprecate EnumChunk trait
    #[color_type(0)]
    ColorType0 { gray: u16 },
    #[color_type(2)]
    ColorType2 { red: u16, green: u16, blue: u16 },
    #[color_type(3)]
    ColorType3 { palette_index: u8 },
    #[color_type(4)]
    ColorType4 { gray: u16 },
    #[color_type(6)]
    ColorType6 { red: u16, green: u16, blue: u16 },
}

#[derive(Debug, ChunkData)]
struct gAMA {
    image_gamma: u32,
}

impl gAMA {
    fn image_gamma(&self) -> f64 {
        self.image_gamma as f64 / DIVISOR
    }
}

// this reflects the palette chunk
// must be between plte and first idat
#[derive(Debug, ChunkData)]
struct hIST {
    entries: Vec<u16>,
}

#[derive(Debug, ChunkData)]
pub struct pHYs {
    // pixels per unit x axis
    x_ppu: u32,
    y_ppu: u32,
    unit_specifier: u8,
}

#[derive(Debug, ChunkData)]
pub struct tEXt {
    // #[len(>= 1, <= 79)]
    #[delimiter(0)]
    #[color_type(2)]
    keyword: Vec<u8>,
    text: Vec<u8>,
}

impl tEXt {
    pub fn keyword(&self) -> &[u8] {
        &self.keyword
    }

    pub fn text(&self) -> &[u8] {
        &self.text
    }

    pub fn parse(&self) -> [std::borrow::Cow<'_, str>; 2] {
        [
            String::from_utf8_lossy(self.keyword()),
            String::from_utf8_lossy(self.text()),
        ]
    }
}

#[derive(Debug, ChunkData)]
pub struct IEND {}

#[derive(Debug, ChunkData)]
enum sBIT {
    #[color_type(0)]
    ColorType0(u8),
    #[color_type(2)]
    ColorType2 { r: u8, g: u8, b: u8 },
    #[color_type(3)]
    ColorType3 { r: u8, g: u8, b: u8 },
    #[color_type(4)]
    ColorType4 { grayscale: u8, alpha: u8 },
    #[color_type(6)]
    ColorType6 { r: u8, g: u8, b: u8, a: u8 },
}

fn stream_octets_to_u32(octets: &mut Iter<u8>) -> u32 {
    (0..4)
        .into_iter()
        .map(|_| octets.next().unwrap())
        .fold(0u32, |mut mask, b| {
            mask <<= 8;
            mask |= *b as u32;

            mask
        })
}

fn stream_octets_to_u64(octets: &mut Iter<u8>) -> u64 {
    (0..8)
        .into_iter()
        .map(|_| octets.next().unwrap())
        .fold(0u64, |mut mask, b| {
            mask <<= 8;
            mask |= *b as u64;

            mask
        })
}

fn stream_octets_to_u16(octets: &mut Iter<u8>) -> u16 {
    (0..2)
        .into_iter()
        .map(|_| octets.next().unwrap())
        .fold(0u16, |mut mask, b| {
            mask <<= 8;
            mask |= *b as u16;

            mask
        })
}

fn stream_vecu8(octets: &mut Iter<u8>, delim: Option<u8>) -> Vec<u8> {
    match delim {
        Some(delim) => octets.take_while(|b| **b != delim).map(|b| *b).collect(),
        None => octets.map(|b| *b).collect(),
    }
}

fn stream_vecu16(octets: &mut Iter<u8>, delim: Option<u8>) -> Vec<u16> {
    match delim {
        Some(delim) => octets
            .take_while(|b| **b != delim)
            .collect::<Vec<&u8>>()
            .chunks(2)
            .map(|b| {
                let mut mask = 0u16;
                mask |= *b[0] as u16;
                mask <<= 8;
                mask |= *b[1] as u16;

                mask
            })
            .collect::<Vec<u16>>(),
        None => octets
            .collect::<Vec<&u8>>()
            .chunks(2)
            .map(|b| {
                let mut mask = 0u16;
                mask |= *b[0] as u16;
                mask <<= 8;
                mask |= *b[1] as u16;

                mask
            })
            .collect::<Vec<u16>>(),
    }
}
