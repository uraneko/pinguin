use ebony::{IHDR, IHDRData, dump_raw, validate_signature};

fn main() {
    let raw = dump_raw("test.png");
    println!("{:?}", &raw[..20]);
    let res = validate_signature(raw);
    if let Err(e) = res {
        panic!("error {:?}", e);
    } else {
        println!(
            "<<{}>>",
            [137, 80, 78, 71, 13, 10, 26, 10]
                .into_iter()
                .map(|b| b as u8 as char)
                .collect::<String>()
        )
    }
    let mut iter = res.unwrap();
    let ihdr = IHDR::from_iter(&mut iter);
    println!("{}", ihdr);
    let data = IHDRData::from_data(ihdr.data());
    println!("{:#?}", data);
}
