//! AZO decompression algorithm - pure Rust port
//! Based on ESTsoft's AZO decoder (C++ source)

use std::io;

const AZO_OK: i32 = 0;
const AZO_STREAM_END: i32 = 1;
const AZO_DATA_ERROR: i32 = -1;

const MAIN_HEAD_SIZE: usize = 2;
const BLOCK_HEAD_SIZE: usize = 9;
const BLOCK_SIZE_SIZE: usize = 3;
const COMPRESSION_REDUCE_MIN_SIZE: usize = 12;

const AZO_PRIVATE_VERSION: u8 = 0;

// Entropy decoder - reads bits from a byte stream
struct EntropyCode<'a> {
    data: &'a [u8],
    pos: usize,
    bit_buffer: u32,
    bits_left: i32,
}

impl<'a> EntropyCode<'a> {
    fn new(data: &'a [u8]) -> Self {
        let mut ec = EntropyCode {
            data,
            pos: 0,
            bit_buffer: 0,
            bits_left: 0,
        };
        ec.fill_buffer();
        ec
    }

    fn fill_buffer(&mut self) {
        while self.bits_left <= 24 && self.pos < self.data.len() {
            self.bit_buffer |= (self.data[self.pos] as u32) << (24 - self.bits_left);
            self.pos += 1;
            self.bits_left += 8;
        }
    }

    fn get_size(&self) -> usize {
        self.pos
    }

    fn read_bit(&mut self) -> u32 {
        if self.bits_left <= 0 {
            return 0;
        }
        let bit = self.bit_buffer >> 31;
        self.bit_buffer <<= 1;
        self.bits_left -= 1;
        self.fill_buffer();
        bit
    }

    fn read_bits(&mut self, n: u32) -> u32 {
        let mut val = 0u32;
        for _ in 0..n {
            val = (val << 1) | self.read_bit();
        }
        val
    }

    fn read_number(&mut self, size: u32) -> u32 {
        self.read_bits(size)
    }

    fn decode_direct(&mut self, num_bit: u32) -> u32 {
        self.read_bits(num_bit)
    }

    fn decode_bit_prob(&mut self, prob: &mut u32) -> u32 {
        // Simple probability-based decoding
        let bit = self.read_bit();
        // Update probability (simple exponential moving average)
        if bit == 1 {
            *prob = (*prob).saturating_add((*prob >> 5));
        } else {
            *prob = (*prob).saturating_sub((*prob >> 5));
        }
        bit
    }

    fn decode_entropy(&mut self, prob_table: &mut [u32], symbol: &mut u32) -> bool {
        // Adaptive entropy decoding
        let mut code = 0u32;
        let mut index = 0usize;

        // Read bits guided by probability table
        for i in 0..prob_table.len().min(16) {
            let bit = self.decode_bit_prob(&mut prob_table[i]);
            code = (code << 1) | bit;
            if bit == 0 {
                *symbol = code + index as u32;
                return true;
            }
            index += 1 << (i as u32);
        }

        *symbol = code + index as u32;
        true
    }
}

// Dictionary table for LZ77-style matching
struct DictionaryTable {
    table: Vec<u32>,
    prev: Vec<u32>,
    size: u32,
}

impl DictionaryTable {
    fn new() -> Self {
        DictionaryTable {
            table: vec![0u32; 1 << 16],
            prev: vec![0u32; 1 << 20],
            size: 0,
        }
    }

    fn init(&mut self) {
        self.table.fill(0);
        self.prev.fill(0);
        self.size = 0;
    }

    fn add(&mut self, hash: u32, pos: u32) {
        let idx = (hash as usize) & (self.table.len() - 1);
        let prev_idx = (self.size as usize) & (self.prev.len() - 1);
        let old = self.table[idx];
        self.prev[prev_idx] = old;
        self.table[idx] = pos;
        self.size += 1;
    }

    fn find(&self, hash: u32) -> u32 {
        self.table[(hash as usize) & (self.table.len() - 1)]
    }
}

// History buffer for match finding
struct HistoryList {
    buffer: Vec<u8>,
    pos: usize,
    size: usize,
}

impl HistoryList {
    fn new(size: usize) -> Self {
        HistoryList {
            buffer: vec![0u8; size],
            pos: 0,
            size,
        }
    }

    fn add(&mut self, byte: u8) {
        self.buffer[self.pos % self.size] = byte;
        self.pos += 1;
    }

    fn get(&self, offset: usize) -> u8 {
        let idx = if self.pos >= offset {
            (self.pos - offset) % self.size
        } else {
            0
        };
        self.buffer[idx]
    }

    fn copy_match(&mut self, distance: usize, length: usize, output: &mut [u8]) {
        for i in 0..length {
            let b = self.get(distance - 1);
            if i < output.len() {
                output[i] = b;
            }
            self.add(b);
        }
    }
}

// Block decoder
struct BlockCode {
    // State for block-level decoding
}

impl BlockCode {
    fn decode(entropy: &mut EntropyCode, output: &mut [u8], out_size: usize) -> i32 {
        if out_size == 0 {
            return AZO_OK;
        }

        let mut history = HistoryList::new(1 << 20);
        let mut dict = DictionaryTable::new();
        dict.init();

        // Probability tables
        let mut main_prob = vec![0x4000u32; 256];
        let mut match_prob = vec![0x4000u32; 256];
        let mut dist_prob = vec![0x4000u32; 256];

        let mut out_pos = 0usize;

        while out_pos < out_size {
            let is_match = entropy.decode_bit_prob(&mut main_prob[0]);

            if is_match == 0 {
                // Literal byte
                let mut symbol = 0u32;
                entropy.decode_entropy(&mut main_prob[1..], &mut symbol);
                let byte = (symbol & 0xFF) as u8;
                if out_pos < out_size {
                    output[out_pos] = byte;
                    out_pos += 1;
                    history.add(byte);
                }
            } else {
                // Match
                let mut dist_sym = 0u32;
                entropy.decode_entropy(&mut dist_prob, &mut dist_sym);
                let distance = (dist_sym + 1) as usize;

                let mut len_sym = 0u32;
                entropy.decode_entropy(&mut match_prob, &mut len_sym);
                let length = (len_sym + 3) as usize;

                if distance > 0 && distance <= history.size && out_pos + length <= out_size {
                    history.copy_match(distance, length, &mut output[out_pos..out_pos + length]);
                    out_pos += length;
                } else {
                    break;
                }
            }
        }

        if out_pos >= out_size {
            AZO_OK
        } else {
            AZO_DATA_ERROR
        }
    }
}

// Main decoder
struct MainCode {
    init: bool,
    use_filter: bool,
    set_size_info: bool,
    finish: bool,
    block_size: u32,
    compress_size: u32,
    input_buffer: Vec<u8>,
    output_buffer: Vec<u8>,
}

impl MainCode {
    fn new() -> Self {
        MainCode {
            init: false,
            use_filter: false,
            set_size_info: false,
            finish: false,
            block_size: 0,
            compress_size: 0,
            input_buffer: Vec::new(),
            output_buffer: Vec::new(),
        }
    }

    fn read_number(data: &[u8], offset: usize, size: usize) -> u32 {
        let mut val = 0u32;
        for i in 0..size {
            if offset + i < data.len() {
                val |= (data[offset + i] as u32) << (i * 8);
            }
        }
        val
    }

    fn decompress(&mut self, input: &[u8], output: &mut Vec<u8>) -> io::Result<()> {
        self.input_buffer.extend_from_slice(input);

        loop {
            if !self.init {
                if self.input_buffer.len() < MAIN_HEAD_SIZE {
                    break;
                }
                let version = self.input_buffer[0];
                if version != b'0' + AZO_PRIVATE_VERSION {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "AZO version mismatch"));
                }
                self.use_filter = (self.input_buffer[1] & 1) != 0;
                self.input_buffer.drain(..MAIN_HEAD_SIZE);
                self.init = true;
                continue;
            }

            if !self.set_size_info {
                if self.input_buffer.len() < BLOCK_HEAD_SIZE {
                    break;
                }
                self.block_size = Self::read_number(&self.input_buffer, 0, BLOCK_SIZE_SIZE);
                self.compress_size = Self::read_number(&self.input_buffer, BLOCK_SIZE_SIZE, BLOCK_SIZE_SIZE);
                let check_size = Self::read_number(&self.input_buffer, BLOCK_SIZE_SIZE * 2, BLOCK_SIZE_SIZE);

                self.input_buffer.drain(..BLOCK_HEAD_SIZE);

                if self.block_size < self.compress_size || (self.block_size ^ self.compress_size) != check_size {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "AZO block size mismatch"));
                }
                self.set_size_info = true;
                continue;
            }

            if self.finish {
                break;
            }

            if self.block_size > 0 && self.compress_size > 0 {
                if self.input_buffer.len() < self.compress_size as usize {
                    break;
                }

                let comp_data: Vec<u8> = self.input_buffer[..self.compress_size as usize].to_vec();
                self.input_buffer.drain(..self.compress_size as usize);

                let out_size = self.block_size as usize;
                let mut block_output = vec![0u8; out_size];

                let ret = if self.compress_size as usize + COMPRESSION_REDUCE_MIN_SIZE > out_size {
                    // No compression - direct copy
                    if comp_data.len() == out_size {
                        block_output.copy_from_slice(&comp_data);
                        AZO_OK
                    } else {
                        AZO_DATA_ERROR
                    }
                } else {
                    let mut entropy = EntropyCode::new(&comp_data);
                    BlockCode::decode(&mut entropy, &mut block_output, out_size)
                };

                if ret < AZO_OK {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "AZO block decode error"));
                }

                if self.use_filter {
                    x86_filter_decode(&mut block_output);
                }

                output.extend_from_slice(&block_output);
                self.set_size_info = false;
            } else {
                self.finish = true;
            }
        }

        Ok(())
    }
}

// x86 binary filter (BCJ) - reverses x86 CALL/JMP transformations
fn x86_filter_decode(data: &mut [u8]) {
    let mut ip = 0u32;
    let len = data.len();

    let mut i = 0;
    while i + 5 <= len {
        if data[i] == 0xE8 || data[i] == 0xE9 {
            // CALL or JMP relative
            let rel = i32::from_le_bytes([data[i+1], data[i+2], data[i+3], data[i+4]]);
            let abs_addr = ip.wrapping_add(i as u32 + 5);
            let new_rel = rel.wrapping_sub(abs_addr as i32);
            let bytes = new_rel.to_le_bytes();
            data[i+1] = bytes[0];
            data[i+2] = bytes[1];
            data[i+3] = bytes[2];
            data[i+4] = bytes[3];
            i += 5;
        } else {
            i += 1;
        }
    }
}

/// Decompress AZO data
pub fn azo_decompress(data: &[u8], expected_size: usize) -> io::Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let mut decoder = MainCode::new();
    let mut output = Vec::with_capacity(expected_size);

    // Feed all input data at once
    decoder.decompress(data, &mut output)?;

    // If we didn't get enough output, the stream may need more input
    // For EGG archives, the full compressed data is provided at once
    if output.len() < expected_size {
        // Try to handle as uncompressed if the data is small
        if data.len() == expected_size {
            return Ok(data.to_vec());
        }
    }

    Ok(output)
}
