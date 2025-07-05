// rfc2083
// png version 1.0
use std::vec::IntoIter;

mod chunks;
mod data;
mod idat;
mod zlib;

pub use zlib::ZLib;

pub use chunks::Chunk;

pub use data::{
    IDAT, IEND, IHDR, PLTE, bKGD, cHRM, gAMA, hIST, pHYs, sBIT, tEXt, tIME, tRNS, zTXT,
};

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

#[derive(Debug)]
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

    pub fn into_chunks(self) -> Vec<Chunk> {
        self.state
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

pub struct PNGData {
    ihdr: IHDR,
    // MUST ct 3
    // COULD ct 2, 6
    // MUST NOT ct 0
    // can there only be 1
    // number of entries <= 2 ^ (ihdr bit depth)
    plte: Option<PLTE>,
    bkgd: Option<bKGD>,
    chrm: Option<cHRM>,
    gama: Option<gAMA>,
    hist: Option<hIST>,
    phys: Option<pHYs>,
    sbit: Option<sBIT>,
    time: Option<tIME>,
    trns: Option<tRNS>,
    text: Option<Vec<tEXt>>,
    ztxt: Option<Vec<zTXT>>,
    idat: Vec<IDAT>,
    iend: IEND,
}

impl PNGData {
    fn with_ihdr_iend(ihdr: Chunk, iend: Chunk) -> Self {
        Self {
            ihdr: ihdr.chunk_data::<IHDR>(None).unwrap(),
            iend: iend.chunk_data::<IEND>(None).unwrap(),
            idat: Vec::default(),
            plte: Option::default(),
            bkgd: Option::default(),
            chrm: Option::default(),
            gama: Option::default(),
            hist: Option::default(),
            phys: Option::default(),
            sbit: Option::default(),
            text: Option::default(),
            time: Option::default(),
            trns: Option::default(),
            ztxt: Option::default(),
        }
    }

    fn ct(&self) -> u8 {
        self.ihdr.color_type()
    }

    pub fn new(chunks: PNGChunks) -> Self {
        let mut chunks = chunks.into_chunks();
        let mut png = Self::with_ihdr_iend(chunks.remove(0), chunks.pop().unwrap());
        let ct = png.ct();
        chunks.into_iter().for_each(|c| match &c.type_name()[..] {
            "PLTE" => png.plte = c.chunk_data::<PLTE>(None).ok(),
            "bKGD" => png.bkgd = c.chunk_data::<bKGD>(Some(ct)).ok(),
            "cHRM" => png.chrm = c.chunk_data::<cHRM>(None).ok(),
            "gAMA" => png.gama = c.chunk_data::<gAMA>(None).ok(),
            "hIST" => png.hist = c.chunk_data::<hIST>(None).ok(),
            "pHYs" => png.phys = c.chunk_data::<pHYs>(None).ok(),
            "sBIT" => png.sbit = c.chunk_data::<sBIT>(Some(ct)).ok(),
            "tEXt" => {
                png.text
                    .as_mut()
                    .map(|v| v.push(c.chunk_data::<tEXt>(Some(ct)).unwrap()));
            }
            "tIME" => png.time = c.chunk_data::<tIME>(None).ok(),
            "tRNS" => png.trns = c.chunk_data::<tRNS>(Some(ct)).ok(),
            "zTXT" => {
                png.ztxt
                    .as_mut()
                    .map(|v| v.push(c.chunk_data::<zTXT>(None).unwrap()));
            }
            "IDAT" => png.idat.push(c.chunk_data::<IDAT>(None).unwrap()),
            val => unreachable!("I dont know the {} chunk", val),
        });

        png
    }

    fn idat_stats(&self) -> String {
        format!(
            "\n   len: {},\n   max: {}\n   min{}: {}\n   unq: {:?}\n",
            self.idat.len(),
            self.idat.iter().map(|idat| idat.len()).max().unwrap(),
            {
                if self.idat.iter().map(|idat| idat.len()).min().unwrap()
                    == self.idat.iter().map(|idat| idat.len()).last().unwrap()
                {
                    "(last)"
                } else {
                    ""
                }
            },
            self.idat.iter().map(|idat| idat.len()).min().unwrap(),
            {
                let mut dv = self
                    .idat
                    .iter()
                    .map(|idat| idat.len())
                    .collect::<Vec<usize>>();
                dv.dedup();

                dv
            }
        )
    }

    pub fn data(&self) -> std::slice::Iter<IDAT> {
        self.idat.iter()
    }
}

impl std::fmt::Debug for PNGData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PNGData {{\n   {:#?},\n{}   IDAT {{{}}}}}",
            self.ihdr,
            {
                let mut s = String::new();
                if let Some(ref plte) = self.plte {
                    s.push_str(&format!("   {:#?},\n", plte));
                }
                if let Some(ref bkgd) = self.bkgd {
                    s.push_str(&format!("   bKGD::{:#?},\n", bkgd));
                }
                if let Some(ref chrm) = self.chrm {
                    s.push_str(&format!("   {:#?},\n", chrm));
                }
                if let Some(ref gama) = self.gama {
                    s.push_str(&format!("   {:#?},\n", gama));
                }
                if let Some(ref hist) = self.hist {
                    s.push_str(&format!("   {:#?},\n", hist));
                }
                if let Some(ref phys) = self.phys {
                    s.push_str(&format!("   {:#?},\n", phys));
                }
                if let Some(ref sbit) = self.sbit {
                    s.push_str(&format!("   sBIT::{:#?},\n", sbit));
                }
                if let Some(ref text) = self.text {
                    s.push_str(&format!("   {:#?},\n", text));
                }
                if let Some(ref time) = self.time {
                    s.push_str(&format!("   {:#?},\n", time));
                }
                if let Some(ref trns) = self.trns {
                    s.push_str(&format!("   tRNS::{:#?},\n", trns));
                }
                if let Some(ref ztxt) = self.ztxt {
                    s.push_str(&format!("   {:#?},\n", ztxt));
                }
                s
            },
            self.idat_stats()
        )
    }
}
