//! XOR encryption and checksum utilities for Pokémon sub-structure data.
//!
//! Gen III stores the four data blocks of each Pokémon XOR-encrypted with a 32-bit key
//! derived from `personality_value ^ ot_id`. The same [`crypt_data`] function is used for
//! both encryption and decryption (XOR is self-inverse).
//!
//! The 16-bit [`calculate_checksum`] is computed over the **unencrypted** flat sub-structure
//! bytes and stored in the Pokémon header at offset 0x1C.

use byteorder::ByteOrder;
use byteorder::LittleEndian;

/// XOR-encrypts or decrypts a 48-byte sub-structure buffer in-place.
///
/// Processes the data 4 bytes (one `u32`) at a time, XORing each chunk with `key`.
/// The same call encrypts and decrypts because XOR is self-inverse.
pub fn crypt_data(data: &mut [u8], key: u32) {
    // second we decrypt the data by XORing it, 32 bits (or 4 bytes) at a time with the encryption key
    // We accomplish this by splitting the data substructure in chunks of 4 bytes (u32) and XORing every chunk
    for chunk in data.chunks_mut(4) {
        let val = LittleEndian::read_u32(chunk);
        LittleEndian::write_u32(chunk, val ^ key);
    }
}

/// Calculates the 16-bit checksum over the unencrypted sub-structure bytes.
///
/// Sums all 2-byte (little-endian `u16`) chunks of the 48-byte flat sub-structure with
/// wrapping addition. The result is stored at offset 0x1C of the Pokémon header.
pub fn calculate_checksum(data: &[u8]) -> u16 {
    let mut sum: u16 = 0;
    for chunk in data.chunks(2) {
        sum = sum.wrapping_add(LittleEndian::read_u16(chunk));
    }
    sum
}
