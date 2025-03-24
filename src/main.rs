use ebony::{IHDRData, PNGChunks, ProcessData, RawPNG};

fn main() {
    let mut raw = RawPNG::from_file_name("test.png").unwrap();
    let mut chunks = PNGChunks::from_raw(raw);

    println!("{:#?}", chunks.names());

    let ihdr_data = chunks.chunks().iter().next().unwrap().process();

    println!("{:?}", ihdr_data);
}
