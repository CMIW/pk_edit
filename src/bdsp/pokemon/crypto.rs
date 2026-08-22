//! PB8 (Gen 8 BDSP) Pokémon encryption, decryption, and checksum utilities.
//!
//! The 344-byte party record (or 328-byte stored record) is split into an 8-byte
//! header followed by four 80-byte blocks, optionally followed by 16 bytes of
//! battle stats. Encryption XORs the blocks — and, with a fresh keystream, the
//! trailing battle stats — using a linear congruential generator seeded with the
//! Encryption Constant at offset 0. The four blocks are also permuted according
//! to `(EC >> 13) & 31`.

use byteorder::ByteOrder;
use byteorder::LittleEndian;

pub const SIZE_STORED: usize = 0x148;
pub const SIZE_PARTY: usize = 0x158;
pub const SIZE_BLOCK: usize = 0x50;

const HEADER_LEN: usize = 8;
const BLOCKS_END: usize = HEADER_LEN + (4 * SIZE_BLOCK);

/// Permutation of the four blocks for each of the 32 possible shuffle values.
///
/// The final eight rows duplicate rows 0–7 so that `sv` (which ranges 0–31) can
/// index the table directly without a modulus.
#[rustfmt::skip]
const BLOCK_POSITION: [u8; 128] = [
    0,1,2,3, 0,1,3,2, 0,2,1,3, 0,3,1,2, 0,2,3,1, 0,3,2,1,
    1,0,2,3, 1,0,3,2, 2,0,1,3, 3,0,1,2, 2,0,3,1, 3,0,2,1,
    1,2,0,3, 1,3,0,2, 2,1,0,3, 3,1,0,2, 2,3,0,1, 3,2,0,1,
    1,2,3,0, 1,3,2,0, 2,1,3,0, 3,1,2,0, 2,3,1,0, 3,2,1,0,
    0,1,2,3, 0,1,3,2, 0,2,1,3, 0,3,1,2, 0,2,3,1, 0,3,2,1,
    1,0,2,3, 1,0,3,2,
];

/// Maps a shuffle value to the shuffle value that reverses it.
///
/// Used when encrypting: the resulting value is fed back into [`BLOCK_POSITION`].
/// The final eight rows duplicate rows 0–7 so that `sv` (which ranges 0–31) can
/// index the table directly, mirroring [`BLOCK_POSITION`].
#[rustfmt::skip]
const BLOCK_POSITION_INVERT: [u8; 32] = [
    0,1,2,4,3,5,6,7,12,18,13,19,8,10,14,20,16,22,9,11,15,21,17,23,
    0,1,2,4,3,5,6,7,
];

fn read_u16_at(data: &[u8], offset: usize) -> u16 {
    data.get(offset..offset.saturating_add(2))
        .map(LittleEndian::read_u16)
        .unwrap_or(0)
}

fn read_u32_at(data: &[u8], offset: usize) -> u32 {
    data.get(offset..offset.saturating_add(4))
        .map(LittleEndian::read_u32)
        .unwrap_or(0)
}

/// Sums little-endian `u16` words from offset 8 up to [`SIZE_STORED`] with
/// wrapping addition. Returns 0 if `data` is too short.
pub fn calculate_checksum(data: &[u8]) -> u16 {
    let Some(slice) = data.get(HEADER_LEN..SIZE_STORED) else {
        return 0;
    };
    let mut sum: u16 = 0;
    for chunk in slice.chunks_exact(2) {
        sum = sum.wrapping_add(LittleEndian::read_u16(chunk));
    }
    sum
}

/// XORs `data` in-place with the LCG keystream derived from `seed`.
///
/// The LCG advances once per `u16` word: `seed = (seed * 0x41C64E6D) + 0x6073`,
/// and the high 16 bits of the updated seed are XORed into the word.
fn crypt_array(data: &mut [u8], seed: u32) {
    let mut state = seed;
    for chunk in data.chunks_exact_mut(2) {
        state = state.wrapping_mul(0x41C6_4E6D).wrapping_add(0x0000_6073);
        let key = (state >> 16) as u16;
        let value = LittleEndian::read_u16(chunk);
        LittleEndian::write_u16(chunk, value ^ key);
    }
}

/// XOR-crypts the four data blocks and, separately, the trailing battle stats.
///
/// Both regions restart the keystream from `seed`, matching the game's routine.
/// The call is self-inverse because XOR is self-inverse.
pub fn crypt_pkm(data: &mut [u8], seed: u32) {
    if let Some(blocks) = data.get_mut(HEADER_LEN..BLOCKS_END) {
        crypt_array(blocks, seed);
    }
    if let Some(stats) = data.get_mut(BLOCKS_END..) {
        crypt_array(stats, seed);
    }
}

/// Reorders the four 80-byte blocks using the permutation selected by `sv`.
///
/// The header and any trailing battle stats are copied verbatim. Returns an
/// empty vector if `data` is shorter than the four blocks.
pub fn shuffle_array(data: &[u8], sv: u32) -> Vec<u8> {
    if data.len() < BLOCKS_END {
        return Vec::new();
    }
    let base = (sv as usize).saturating_mul(4);

    let mut out = data.to_vec();

    for block in 0..4usize {
        let Some(&source) = BLOCK_POSITION.get(base.saturating_add(block)) else {
            return Vec::new();
        };
        let src_start = HEADER_LEN.saturating_add((source as usize).saturating_mul(SIZE_BLOCK));
        let dst_start = HEADER_LEN.saturating_add(block.saturating_mul(SIZE_BLOCK));

        let Some(chunk) = data
            .get(src_start..src_start.saturating_add(SIZE_BLOCK))
            .map(<[u8]>::to_vec)
        else {
            return Vec::new();
        };
        let Some(target) = out.get_mut(dst_start..dst_start.saturating_add(SIZE_BLOCK)) else {
            return Vec::new();
        };
        target.copy_from_slice(&chunk);
    }

    out
}

/// Decrypts a PB8 buffer: XOR-decrypts with the EC-seeded keystream, then
/// restores the four blocks to their canonical order.
pub fn decrypt(data: &[u8]) -> Vec<u8> {
    if data.len() < BLOCKS_END {
        return Vec::new();
    }
    let ec = read_u32_at(data, 0);
    let mut buffer = data.to_vec();
    crypt_pkm(&mut buffer, ec);
    shuffle_array(&buffer, (ec >> 13) & 31)
}

/// Encrypts a PB8 buffer: permutes the four blocks into their scrambled order,
/// then XOR-encrypts with the EC-seeded keystream.
pub fn encrypt(data: &[u8]) -> Vec<u8> {
    if data.len() < BLOCKS_END {
        return Vec::new();
    }
    let ec = read_u32_at(data, 0);
    let sv = (ec >> 13) & 31;
    let Some(&inverted) = BLOCK_POSITION_INVERT.get(sv as usize) else {
        return Vec::new();
    };

    let mut buffer = shuffle_array(data, u32::from(inverted));
    if buffer.is_empty() {
        return Vec::new();
    }
    crypt_pkm(&mut buffer, ec);
    buffer
}

/// Heuristically detects whether `data` is currently encrypted by checking two
/// offsets that are always zero in a decrypted PB8 record.
pub fn is_encrypted(data: &[u8]) -> bool {
    read_u16_at(data, 0x70) != 0 || read_u16_at(data, 0x110) != 0
}
