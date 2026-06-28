pub mod egg;
pub mod alz;

use std::io::{self, Read, Seek, SeekFrom};

pub const MAGIC_EGG: u32 = 0x41474745;
pub const MAGIC_FILE: u32 = 0x0a8590e3;
pub const MAGIC_BLOCK: u32 = 0x02b50c13;
pub const MAGIC_ENCRYPT: u32 = 0x08d1470f;
pub const MAGIC_WINDOWS: u32 = 0x2c86950b;
pub const MAGIC_POSIX: u32 = 0x1ee922e5;
pub const MAGIC_FILENAME: u32 = 0x0a8591ac;
pub const MAGIC_COMMENT: u32 = 0x04c63672;
pub const MAGIC_SPLIT: u32 = 0x24f5a262;
pub const MAGIC_SOLID: u32 = 0x24e5a060;
pub const MAGIC_DUMMY: u32 = 0x07463307;
pub const MAGIC_END: u32 = 0x08e28222;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMethod {
    Store,
    Deflate,
    Bzip2,
    Azo,
    Lzma,
}

impl CompressionMethod {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Store),
            1 => Some(Self::Deflate),
            2 => Some(Self::Bzip2),
            3 => Some(Self::Azo),
            4 => Some(Self::Lzma),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Store => "STORE",
            Self::Deflate => "DEFLATE",
            Self::Bzip2 => "BZIP2",
            Self::Azo => "AZO",
            Self::Lzma => "LZMA",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptMethod {
    None,
    ZipCompatible,
    Aes128,
    Aes256,
}

#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub method: CompressionMethod,
    pub method_hint: u32,
    pub uncomp_size: u32,
    pub comp_size: u32,
    pub crc: u32,
    pub data_offset: u64,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub id: u32,
    pub uncomp_size: u64,
    pub name: String,
    pub packed_size: u64,
    pub blocks: Vec<BlockInfo>,
    pub is_encrypted: bool,
    pub encrypt_method: EncryptMethod,
    pub comment: Option<String>,
    pub win_attrs: Option<u32>,
    pub posix_mode: Option<u32>,
    pub modified: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ArchiveInfo {
    pub version_major: u8,
    pub version_minor: u8,
    pub volume_id: u32,
    pub is_solid: bool,
    pub is_spanned: bool,
    pub files: Vec<FileInfo>,
    pub comment: Option<String>,
}

pub trait ArchiveFormat {
    fn open<R: Read + Seek>(reader: &mut R) -> io::Result<ArchiveInfo>;
    fn extract_file<R: Read + Seek>(
        reader: &mut R,
        file: &FileInfo,
        password: Option<&str>,
    ) -> io::Result<Vec<u8>>;
}

pub fn read_u16_le<R: Read>(r: &mut R) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

pub fn read_u32_le<R: Read>(r: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn read_u64_le<R: Read>(r: &mut R) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}
