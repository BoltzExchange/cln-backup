use anyhow::Result;

#[cfg(feature = "gzip")]
use std::io::Write;

pub trait Compression {
    fn file_suffix(&self) -> &'static str;
    fn compress(&self, data: Vec<u8>) -> Result<Vec<u8>>;
}

#[derive(Clone)]
pub struct NoCompression {}

#[cfg(feature = "gzip")]
#[derive(Clone)]
pub struct Gzip {}

#[cfg(feature = "zstd")]
#[derive(Clone)]
pub struct Zstd {}

impl NoCompression {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "gzip")]
impl Gzip {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "zstd")]
impl Zstd {
    pub fn new() -> Self {
        Self {}
    }
}

impl Compression for NoCompression {
    fn file_suffix(&self) -> &'static str {
        ""
    }

    fn compress(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        Ok(data)
    }
}

#[cfg(feature = "gzip")]
impl Compression for Gzip {
    fn file_suffix(&self) -> &'static str {
        "gz"
    }

    fn compress(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
        encoder.write_all(&data)?;
        Ok(encoder.finish()?)
    }
}

#[cfg(feature = "zstd")]
impl Compression for Zstd {
    fn file_suffix(&self) -> &'static str {
        "zst"
    }

    fn compress(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        Ok(zstd::encode_all(
            &data[..],
            zstd::DEFAULT_COMPRESSION_LEVEL,
        )?)
    }
}

#[derive(Clone)]
pub enum CompressionAlgorithm {
    NoCompression(NoCompression),
    #[cfg(feature = "gzip")]
    Gzip(Gzip),
    #[cfg(feature = "zstd")]
    Zstd(Zstd),
}

impl Compression for CompressionAlgorithm {
    fn file_suffix(&self) -> &'static str {
        match self {
            CompressionAlgorithm::NoCompression(c) => c.file_suffix(),
            #[cfg(feature = "gzip")]
            CompressionAlgorithm::Gzip(c) => c.file_suffix(),
            #[cfg(feature = "zstd")]
            CompressionAlgorithm::Zstd(c) => c.file_suffix(),
        }
    }

    fn compress(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        match self {
            CompressionAlgorithm::NoCompression(c) => c.compress(data),
            #[cfg(feature = "gzip")]
            CompressionAlgorithm::Gzip(c) => c.compress(data),
            #[cfg(feature = "zstd")]
            CompressionAlgorithm::Zstd(c) => c.compress(data),
        }
    }
}

pub fn create_compression(
    algorithm: crate::config::CompressionType,
) -> Result<CompressionAlgorithm> {
    match algorithm {
        #[cfg(feature = "gzip")]
        crate::config::CompressionType::Gzip => Ok(CompressionAlgorithm::Gzip(Gzip::new())),
        #[cfg(not(feature = "gzip"))]
        crate::config::CompressionType::Gzip => Err(anyhow::anyhow!(
            "gzip compression requested but gzip feature not enabled"
        )),
        #[cfg(feature = "zstd")]
        crate::config::CompressionType::Zstd => Ok(CompressionAlgorithm::Zstd(Zstd::new())),
        #[cfg(not(feature = "zstd"))]
        crate::config::CompressionType::Zstd => Err(anyhow::anyhow!(
            "zstd compression requested but zstd feature not enabled"
        )),
        crate::config::CompressionType::None => {
            Ok(CompressionAlgorithm::NoCompression(NoCompression::new()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_compression() {
        let data = "some text";

        let compressed = NoCompression::new()
            .compress(data.as_bytes().to_vec())
            .unwrap();

        assert_eq!(data.as_bytes(), compressed.as_slice());
        assert_eq!(NoCompression {}.file_suffix(), "");
    }

    #[cfg(feature = "gzip")]
    #[test]
    fn gzip_compress() {
        use std::io::Read;

        let data = "some text";

        let compressed = Gzip::new().compress(data.as_bytes().to_vec()).unwrap();

        let mut decoder = flate2::read::GzDecoder::new(&compressed[..]);
        let mut s = String::new();
        decoder.read_to_string(&mut s).unwrap();

        assert_eq!(data, s);
    }

    #[cfg(feature = "zstd")]
    #[test]
    fn zstd_compress() {
        let data = "some text";

        let compressed = Zstd::new().compress(data.as_bytes().to_vec()).unwrap();

        let decompressed = zstd::decode_all(&compressed[..]).unwrap();
        let s = String::from_utf8(decompressed).unwrap();

        assert_eq!(data, s);
    }
}
