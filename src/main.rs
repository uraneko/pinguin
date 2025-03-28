use ebony::{IHDR, bkGD, cHRM, pHYs, tEXt};
use ebony::{PNGChunks, RawPNG};

const TEST_FILE: &str = "test3.png";

fn main() {
    let mut raw = RawPNG::from_file_name(TEST_FILE).unwrap();
    let mut chunks = PNGChunks::from_raw(raw);

    println!("{:?}", {
        let mut cn = chunks.names();
        cn.dedup();
        cn
    });
    let chunks = chunks.chunks();
    println!("{:#?}", chunks[0].chunk_data::<IHDR>(None).unwrap());
    let ct = 2;
    println!("{:#?}", chunks[1].chunk_data::<cHRM>(None).unwrap());
    println!(
        "bkGD::{:#?}",
        chunks[2].chunk_data::<bkGD>(Some(ct)).unwrap()
    );
    println!("{:#?}", chunks[3].chunk_data::<pHYs>(None).unwrap());
    let text = chunks[chunks.len() - 2].chunk_data::<tEXt>(None).unwrap();
    println!("{:?}", text.parse());

    // let ihdr_data: IHDRData = chunks.chunks().iter().next().unwrap().process().unwrap();
    // println!("{:#?}", ihdr_data);
    //
    // let chrm_data: cHRMData = chunks.chunks().iter().next().unwrap().process().unwrap();
    // println!("{:#?}", chrm_data);
}
