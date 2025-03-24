use ebony::{IHDRData, PNGChunks, RawPNG};

fn main() {
    let mut raw = RawPNG::from_file_name("test.png").unwrap();
    let mut chunks = PNGChunks::from_raw(raw);

    println!("{:#?}", chunks.names())
}
