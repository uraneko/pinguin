#[derive(Debug)]
pub struct ZLib {
    // aka flags_code
    compression_method: u8,
    // aka additional_flags
    check_bits: u8,
    compressed_data: Vec<u8>,
    check_value: u32,
}

impl ZLib {
    pub fn from_stream(mut stream: impl std::iter::DoubleEndedIterator<Item = u8>) -> Self {
        let compression_method = stream.next().unwrap();
        let check_bits = stream.next().unwrap();
        let check_value = {
            let mut counter = 4;
            let mut mask = 0u32;
            while counter > 0 {
                let next = stream.next_back().unwrap();
                mask |= next as u32;
                mask <<= 8;
                counter -= 1;
            }

            mask
        };

        Self {
            compression_method,
            check_bits,
            check_value,
            compressed_data: stream.collect(),
        }
    }

    pub fn crc(&self) -> [u8; 4] {
        let val = self.check_value;
        [
            (val >> 24) as u8,
            (val >> 16) as u8,
            (val >> 8) as u8,
            val as u8,
        ]
    }
}

impl std::fmt::Display for ZLib {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ZLib {{\n   compression_method: {},\n   check_bits: {},\n   compressed_data(len): {},\n   check_value: {:?}}}",
            self.compression_method,
            self.check_bits,
            self.compressed_data.len(),
            self.crc()
        )
    }
}

