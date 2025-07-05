use ebony_png::ZLib;
use ebony_png::{IDAT, IHDR, bKGD, cHRM, pHYs, tEXt};
use ebony_png::{PNGChunks, PNGData, RawPNG};

const TEST_FILE: &str = "samples/test3.png";

// TODO all the chunks and all that shouldnt be exposed to a user of the lib
// actually expose fine control of chunks with a feature
// otherwise

fn main() {
    let mut raw = RawPNG::from_file_name(TEST_FILE).unwrap();
    let mut chunks = PNGChunks::from_raw(raw);
    // println!(
    //     "{:#?}",
    //     &chunks.chunks()[4].chunk_data::<IDAT>(None).unwrap().data()[..6]
    // );

    println!("{:?}", {
        let mut d = chunks.names();
        d.dedup();
        d
    });

    let mut chunks = PNGData::new(chunks);
    println!("{:#?}", chunks);

    let data = chunks.concat();
    let compressed = ZLib::from_stream(data);
    println!("{}", compressed);
}
