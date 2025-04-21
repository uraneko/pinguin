use ebony::{IDAT, IHDR, bKGD, cHRM, pHYs, tEXt};
use ebony::{PNGChunks, PNGData, RawPNG};

const TEST_FILE: &str = "samples/test3.png";

// TODO all the chunks and all that shouldnt be exposed to a user of the lib
// actually expose fine control of chunks with a feature
// otherwise

fn main() {
    let mut raw = RawPNG::from_file_name(TEST_FILE).unwrap();
    let mut chunks = PNGChunks::from_raw(raw);
    panic!(
        "{:#?}",
        chunks
            .chunks()
            .into_iter()
            .map(|c| c.crc())
            .collect::<Vec<[u8; 4]>>()
    );

    println!("{:?}", {
        let mut d = chunks.names();
        d.dedup();
        d
    });
    let chunks = PNGData::new(chunks);
    println!("{:#?}", chunks);
}
