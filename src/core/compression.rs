use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};

pub fn compress(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

pub fn decompress_stream(data: &[u8]) -> Result<(Vec<u8>, usize), Box<dyn std::error::Error>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok((result, decoder.total_in() as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let original = b"hello world, this is a test string for compression";
        let compressed = compress(original).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(original.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_empty_data() {
        let original = b"";
        let compressed = compress(original).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(original.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_larger_data() {
        let original = b"AAAA".repeat(1000);
        let original = original.as_slice();
        let compressed = compress(original).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(original, decompressed.as_slice());
    }

    #[test]
    fn test_compression_reduces_size() {
        let original = b"AAAA".repeat(1000);
        let compressed = compress(&original).unwrap();
        assert!(compressed.len() < original.len());
    }

    #[test]
    fn test_decompress_invalid_errors() {
        let result = decompress(b"this is not valid zlib data at all");
        assert!(result.is_err());
    }
}
