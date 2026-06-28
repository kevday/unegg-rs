use std::io::{self, Read, Seek};
use super::*;

pub struct AlzFormat;

impl ArchiveFormat for AlzFormat {
    fn open<R: Read + Seek>(_reader: &mut R) -> io::Result<ArchiveInfo> {
        // ALZ format support - simplified implementation
        Err(io::Error::new(io::ErrorKind::Unsupported, "ALZ format not yet implemented"))
    }

    fn extract_file<R: Read + Seek>(
        _reader: &mut R,
        _file: &FileInfo,
        _password: Option<&str>,
    ) -> io::Result<Vec<u8>> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "ALZ format not yet implemented"))
    }
}
