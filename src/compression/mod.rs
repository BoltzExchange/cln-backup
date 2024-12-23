use anyhow::Result;
use flate2::write::GzEncoder;
use std::io::Write;

pub trait Compression {
    fn file_suffix(&self) -> String;
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>>;
}

#[derive(Clone)]
pub struct Gzip {}

impl Compression for Gzip {
    fn file_suffix(&self) -> String {
        "gz".to_string()
    }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::best());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }
}

#[cfg(test)]
mod test {
    use super::{Compression, Gzip};
    use flate2::read::GzDecoder;
    use std::io::Read;

    #[test]
    fn gzip_compress() {
        let data = "some text";

        let compressed = Gzip {}.compress(data.as_bytes()).unwrap();

        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut s = String::new();
        decoder.read_to_string(&mut s).unwrap();

        assert_eq!(data, s);
    }
}
