pub mod azo;

use std::io::{self, Read};
pub use azo::azo_decompress;

pub fn store(data: &[u8]) -> io::Result<Vec<u8>> {
    Ok(data.to_vec())
}

pub fn deflate(data: &[u8]) -> io::Result<Vec<u8>> {
    use flate2::read::DeflateDecoder;
    let mut decoder = DeflateDecoder::new(data);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output)?;
    Ok(output)
}

pub fn bzip2_decompress(data: &[u8]) -> io::Result<Vec<u8>> {
    use bzip2::read::BzDecoder;
    let mut decoder = BzDecoder::new(data);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output)?;
    Ok(output)
}

pub fn lzma_decompress(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut input = std::io::Cursor::new(data);
    let mut output = Vec::new();
    lzma_rs::lzma_decompress(&mut input, &mut output)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("LZMA error: {e}")))?;
    Ok(output)
}
