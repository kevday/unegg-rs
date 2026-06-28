use std::io::{self, Read, Seek, SeekFrom, Cursor};
use super::*;
use crate::algo;

pub struct EggFormat;

impl ArchiveFormat for EggFormat {
    fn open<R: Read + Seek>(reader: &mut R) -> io::Result<ArchiveInfo> {
        // Read global header
        let magic = read_u32_le(reader)?;
        if magic != MAGIC_EGG {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Not an EGG archive"));
        }
        let version = read_u16_le(reader)?;
        let version_major = (version >> 8) as u8;
        let version_minor = (version & 0xFF) as u8;
        let volume_id = read_u32_le(reader)?;
        let _reserved = read_u32_le(reader)?;

        if version_major > 1 {
            return Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported EGG version"));
        }

        let mut info = ArchiveInfo {
            version_major,
            version_minor,
            volume_id,
            is_solid: false,
            is_spanned: false,
            files: Vec::new(),
            comment: None,
        };

        // Scan headers
        let mut current_file: Option<FileInfo> = None;
        let mut current_block: Option<BlockInfo> = None;

        loop {
            let pos = reader.stream_position()?;
            match read_u32_le(reader) {
                Ok(sig) => match sig {
                    MAGIC_FILE => {
                        // Save previous file if any
                        if let Some(f) = current_file.take() {
                            info.files.push(f);
                        }
                        let id = read_u32_le(reader)?;
                        let length = read_u64_le(reader)?;
                        current_file = Some(FileInfo {
                            id,
                            uncomp_size: length,
                            name: String::new(),
                            packed_size: 0,
                            blocks: Vec::new(),
                            is_encrypted: false,
                            encrypt_method: EncryptMethod::None,
                            comment: None,
                            win_attrs: None,
                            posix_mode: None,
                            modified: None,
                        });
                    }
                    MAGIC_BLOCK => {
                        let method = read_u32_le(reader)?;
                        let hint = read_u32_le(reader)?;
                        let uncomp = read_u32_le(reader)?;
                        let comp = read_u32_le(reader)?;
                        let crc = read_u32_le(reader)?;
                        let data_offset = reader.stream_position()?;
                        let block = BlockInfo {
                            method: CompressionMethod::from_u32(method)
                                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("Unknown compression method: {method}")))?,
                            method_hint: hint,
                            uncomp_size: uncomp,
                            comp_size: comp,
                            crc,
                            data_offset,
                        };
                        if let Some(ref mut f) = current_file {
                            f.packed_size += comp as u64;
                            f.blocks.push(block.clone());
                        }
                        // Skip compressed data
                        reader.seek(SeekFrom::Current(comp as i64))?;
                    }
                    MAGIC_FILENAME => {
                        let _id = read_u32_le(reader)?;
                        let name_len = read_u32_le(reader)? as usize;
                        let mut name_buf = vec![0u8; name_len];
                        reader.read_exact(&mut name_buf)?;
                        // Decode EUC-KR to UTF-8
                        let name = decode_euckr(&name_buf);
                        if let Some(ref mut f) = current_file {
                            f.name = name;
                        }
                    }
                    MAGIC_ENCRYPT => {
                        let method = read_u32_le(reader)?;
                        let _crc = read_u32_le(reader)?;
                        let _verify_len = read_u32_le(reader)?;
                        let mut verify = vec![0u8; _verify_len as usize];
                        reader.read_exact(&mut verify)?;
                        if let Some(ref mut f) = current_file {
                            f.is_encrypted = true;
                            f.encrypt_method = match method {
                                0 => EncryptMethod::ZipCompatible,
                                1 => EncryptMethod::Aes128,
                                2 => EncryptMethod::Aes256,
                                _ => EncryptMethod::None,
                            };
                        }
                    }
                    MAGIC_WINDOWS => {
                        let attrs = read_u32_le(reader)?;
                        let modified = read_u64_le(reader)?;
                        if let Some(ref mut f) = current_file {
                            f.win_attrs = Some(attrs);
                            f.modified = Some(modified);
                        }
                    }
                    MAGIC_POSIX => {
                        let mode = read_u32_le(reader)?;
                        let _uid = read_u32_le(reader)?;
                        let _gid = read_u32_le(reader)?;
                        if let Some(ref mut f) = current_file {
                            f.posix_mode = Some(mode);
                        }
                    }
                    MAGIC_COMMENT => {
                        let _id = read_u32_le(reader)?;
                        let comment_len = read_u32_le(reader)? as usize;
                        let mut comment_buf = vec![0u8; comment_len];
                        reader.read_exact(&mut comment_buf)?;
                        let comment = decode_euckr(&comment_buf);
                        if let Some(ref mut f) = current_file {
                            f.comment = Some(comment);
                        } else {
                            info.comment = Some(comment);
                        }
                    }
                    MAGIC_SOLID => {
                        info.is_solid = true;
                        // Read solid field data
                        let len = read_u32_le(reader)? as usize;
                        let mut buf = vec![0u8; len];
                        reader.read_exact(&mut buf)?;
                    }
                    MAGIC_SPLIT => {
                        info.is_spanned = true;
                        let len = read_u32_le(reader)? as usize;
                        let mut buf = vec![0u8; len];
                        reader.read_exact(&mut buf)?;
                    }
                    MAGIC_DUMMY => {
                        let len = read_u32_le(reader)? as usize;
                        let mut buf = vec![0u8; len];
                        reader.read_exact(&mut buf)?;
                    }
                    MAGIC_END => {
                        // Save last file
                        if let Some(f) = current_file.take() {
                            info.files.push(f);
                        }
                        break;
                    }
                    _ => {
                        // Unknown header - try to skip
                        // Look ahead for known magic bytes
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Unknown EGG header magic: 0x{sig:08x} at offset 0x{:x}", pos),
                        ));
                    }
                },
                Err(e) => {
                    // EOF or error - save what we have
                    if let Some(f) = current_file.take() {
                        info.files.push(f);
                    }
                    break;
                }
            }
        }

        Ok(info)
    }

    fn extract_file<R: Read + Seek>(
        reader: &mut R,
        file: &FileInfo,
        password: Option<&str>,
    ) -> io::Result<Vec<u8>> {
        let mut output = Vec::with_capacity(file.uncomp_size as usize);

        for block in &file.blocks {
            reader.seek(SeekFrom::Start(block.data_offset))?;
            let mut comp_data = vec![0u8; block.comp_size as usize];
            reader.read_exact(&mut comp_data)?;

            // Decrypt if needed
            let decrypted = if file.is_encrypted {
                match file.encrypt_method {
                    EncryptMethod::Aes256 => {
                        let pwd = password.ok_or_else(|| {
                            io::Error::new(io::ErrorKind::InvalidInput, "Password required for encrypted archive")
                        })?;
                        crate::crypto::aes_decrypt(&comp_data, pwd)?
                    }
                    EncryptMethod::ZipCompatible => {
                        let pwd = password.ok_or_else(|| {
                            io::Error::new(io::ErrorKind::InvalidInput, "Password required for encrypted archive")
                        })?;
                        crate::crypto::zip_decrypt(&comp_data, pwd)
                    }
                    _ => comp_data,
                }
            } else {
                comp_data
            };

            // Decompress
            let decompressed = match block.method {
                CompressionMethod::Store => algo::store(&decrypted)?,
                CompressionMethod::Deflate => algo::deflate(&decrypted)?,
                CompressionMethod::Bzip2 => algo::bzip2_decompress(&decrypted)?,
                CompressionMethod::Lzma => algo::lzma_decompress(&decrypted)?,
                CompressionMethod::Azo => algo::azo_decompress(&decrypted, block.uncomp_size as usize)?,
            };

            // Verify CRC
            let crc = crc32fast::hash(&decompressed);
            if crc != block.crc {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("CRC mismatch: expected 0x{:08x}, got 0x{:08x}", block.crc, crc),
                ));
            }

            output.extend_from_slice(&decompressed);
        }

        Ok(output)
    }
}

fn decode_euckr(data: &[u8]) -> String {
    let (cow, _encoding_used, _had_errors) = encoding_rs::EUC_KR.decode(data);
    cow.into_owned()
}
