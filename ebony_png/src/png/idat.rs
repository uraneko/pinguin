use super::{IDAT, IHDR, PNGData};

impl PNGData {
    pub fn color_type_size(&self) -> usize {
        match self.ihdr.color_type() {
            0 => 1,
            2 => 3,
            3 => 1,
            4 => 2,
            6 => 4,
            _ => unreachable!(),
        }
    }

    pub fn concat(&mut self) -> impl std::iter::DoubleEndedIterator<Item = u8> {
        let data = std::mem::take(&mut self.idat);

        data.into_iter().map(|idat| idat.data()).flatten()
    }

    // pub fn scanlines(&mut self) -> Vec<Vec<u8>> {
    //     let data = self.concat();
    //
    //     let filter_type = self.ihdr.filter_type();
    //     data.chunks(self.ihdr.width() as usize)
    //         .map(|c| c.to_vec())
    //         .collect()
    // }
}

#[derive(Debug, Clone)]
pub enum ColorType {
    // 0
    Grayscale(u8),
    // 2
    RGB(RGB),
    // 3
    PLTEIndex(u8),
    // 4
    GrayscaleAlpha { gray: u8, alpha: u8 },
    // 6
    RGBAlpha { rgb: RGB, alpha: u8 },
}

#[derive(Debug, Clone)]
pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}
