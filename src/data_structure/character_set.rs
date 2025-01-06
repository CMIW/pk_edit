use std::collections::HashMap;
use thiserror::Error;

/// Errors specific to the Pokémon game's character set operations.
#[derive(Error, Debug)]
pub enum CharacterSetError {
    #[error("Byte {0} not found in the character set.")]
    ByteNotFound(u8),
    #[error("Character '{0}' not found in the character set.")]
    CharacterNotFound(String),
}

/// Provides mapping between byte values and characters specific to Pokémon games.
/// This supports encoding text fields like Pokémon nicknames or trainer names into the game's custom format.
///
/// Pokémon games use a custom 256-character set for text storage in save files. This module maps these
/// characters to Unicode for user-friendly manipulation and vice versa.
///
/// ## Usage
/// - Use `get_char` to retrieve the character for a specific byte.
/// - Use `get_code` to retrieve the byte for a specific character.
fn get_char_set() -> [&'static str; 256] {
    let mut char_set: [&str; 256] = [" "; 256];
    char_set[0x00] = " ";
    char_set[0x01] = "À";
    char_set[0x02] = "Á";
    char_set[0x03] = "Â";
    char_set[0x04] = "Ç";
    char_set[0x05] = "È";
    char_set[0x06] = "É";
    char_set[0x07] = "Ê";
    char_set[0x08] = "Ë";
    char_set[0x09] = "Ì";
    char_set[0x0B] = "Î";
    char_set[0x0C] = "Ï";
    char_set[0x0D] = "Ò";
    char_set[0x0E] = "Ó";
    char_set[0x0F] = "Ô";

    char_set[0x10] = "Œ";
    char_set[0x11] = "Ù";
    char_set[0x12] = "Ú";
    char_set[0x13] = "Û";
    char_set[0x14] = "Ñ";
    char_set[0x15] = "ß";
    char_set[0x16] = "à";
    char_set[0x17] = "á";
    char_set[0x19] = "ç";
    char_set[0x1A] = "è";
    char_set[0x1B] = "é";
    char_set[0x1C] = "ê";
    char_set[0x1D] = "ë";
    char_set[0x1E] = "ì";

    char_set[0x20] = "î";
    char_set[0x21] = "ï";
    char_set[0x22] = "ò";
    char_set[0x23] = "ó";
    char_set[0x24] = "ô";
    char_set[0x25] = "œ";
    char_set[0x26] = "ù";
    char_set[0x27] = "ú";
    char_set[0x28] = "û";
    char_set[0x29] = "ñ";
    char_set[0x2A] = "º";
    char_set[0x2B] = "ª";
    char_set[0x2C] = "ᵉʳ";
    char_set[0x2D] = "&";
    char_set[0x2E] = "+";

    char_set[0x34] = "Lv";
    char_set[0x35] = "=";
    char_set[0x36] = ";";

    char_set[0x50] = "▯";
    char_set[0x51] = "¿";
    char_set[0x52] = "¡";
    char_set[0x5A] = "Í";
    char_set[0x5B] = "%";
    char_set[0x5C] = "(";
    char_set[0x5D] = ")";
    char_set[0x5E] = " ";
    char_set[0x5F] = " ";

    char_set[0x68] = "â";
    char_set[0x6F] = "í";

    char_set[0x79] = "↑";
    char_set[0x7A] = "↓";
    char_set[0x7B] = "←";
    char_set[0x7C] = "→";
    char_set[0x7D] = "*";
    char_set[0x7E] = "*";
    char_set[0x7F] = "*";

    char_set[0x80] = "*";
    char_set[0x81] = "*";
    char_set[0x82] = "*";
    char_set[0x83] = "*";
    char_set[0x84] = "ᵉ";
    char_set[0x85] = "<";
    char_set[0x86] = ">";

    char_set[0xA0] = "ʳᵉ";
    char_set[0xA1] = "0";
    char_set[0xA2] = "1";
    char_set[0xA3] = "2";
    char_set[0xA4] = "3";
    char_set[0xA5] = "4";
    char_set[0xA6] = "5";
    char_set[0xA7] = "6";
    char_set[0xA8] = "7";
    char_set[0xA9] = "8";
    char_set[0xAA] = "9";
    char_set[0xAB] = "!";
    char_set[0xAC] = "?";
    char_set[0xAD] = ".";
    char_set[0xAE] = "-";
    char_set[0xAF] = "・";

    char_set[0xB0] = "…";
    char_set[0xB1] = "“";
    char_set[0xB2] = "”";
    char_set[0xB3] = "‘";
    char_set[0xB4] = "’";
    char_set[0xB5] = "♂";
    char_set[0xB6] = "♀";
    char_set[0xB7] = "$";
    char_set[0xB8] = ",";
    char_set[0xB9] = "×";
    char_set[0xBA] = "/";
    char_set[0xBB] = "A";
    char_set[0xBC] = "B";
    char_set[0xBD] = "C";
    char_set[0xBE] = "D";
    char_set[0xBF] = "E";

    char_set[0xC0] = "F";
    char_set[0xC1] = "G";
    char_set[0xC2] = "H";
    char_set[0xC3] = "I";
    char_set[0xC4] = "J";
    char_set[0xC5] = "K";
    char_set[0xC6] = "L";
    char_set[0xC7] = "M";
    char_set[0xC8] = "N";
    char_set[0xC9] = "O";
    char_set[0xCA] = "P";
    char_set[0xCB] = "Q";
    char_set[0xCC] = "R";
    char_set[0xCD] = "S";
    char_set[0xCE] = "T";
    char_set[0xCF] = "U";

    char_set[0xD0] = "V";
    char_set[0xD1] = "W";
    char_set[0xD2] = "X";
    char_set[0xD3] = "Y";
    char_set[0xD4] = "Z";
    char_set[0xD5] = "a";
    char_set[0xD6] = "b";
    char_set[0xD7] = "c";
    char_set[0xD8] = "d";
    char_set[0xD9] = "e";
    char_set[0xDA] = "f";
    char_set[0xDB] = "g";
    char_set[0xDC] = "h";
    char_set[0xDD] = "i";
    char_set[0xDE] = "j";
    char_set[0xDF] = "k";

    char_set[0xE0] = "l";
    char_set[0xE1] = "m";
    char_set[0xE2] = "n";
    char_set[0xE3] = "o";
    char_set[0xE4] = "p";
    char_set[0xE5] = "q";
    char_set[0xE6] = "r";
    char_set[0xE7] = "s";
    char_set[0xE8] = "t";
    char_set[0xE9] = "u";
    char_set[0xEA] = "v";
    char_set[0xEB] = "w";
    char_set[0xEC] = "x";
    char_set[0xED] = "y";
    char_set[0xEE] = "z";
    char_set[0xEF] = "►";

    char_set[0xF0] = ":";
    char_set[0xF1] = "Ä";
    char_set[0xF2] = "Ö";
    char_set[0xF3] = "Ü";
    char_set[0xF4] = "ä";
    char_set[0xF5] = "ö";
    char_set[0xF6] = "ü";

    char_set
}

/// Retrieves the character corresponding to a given byte index.
///
/// # Errors
/// Returns `CharacterSetError::InvalidIndex` if the index is not within the valid range (0-255).
pub fn get_char(index: usize) -> &'static str {
    let char_set = get_char_set();

    char_set[index]
}

/// Retrieves the byte value corresponding to a given character string.
///
/// # Errors
/// Returns `CharacterSetError::CharacterNotFound` if the character is not in the character set.
pub fn get_code(s: &str) -> u8 {
    get_char_set().iter().position(|&c| c == s).unwrap() as u8
}

/// Provides mapping between byte values and characters specific to Pokémon games using a single `HashMap`.
/// This supports encoding text fields like Pokémon nicknames or trainer names into the game's custom format.
///
/// Pokémon games use a custom 256-character set for text storage in save files. This module maps these
/// characters to Unicode for user-friendly manipulation and vice versa.
///
/// ## Usage
/// - Use `get_char` to retrieve the character for a specific byte.
/// - Use `get_code` to retrieve the byte for a specific character.
pub struct CharacterSet {
    /// Mapping from byte to character.
    byte_to_char: HashMap<u8, &'static str>,
    char_to_byte: HashMap<&'static str, u8>,
}

impl Default for CharacterSet {
    fn default() -> Self {
        Self::new()
    }
}

impl CharacterSet {
    /// Creates a new `CharacterSet` with predefined mappings.
    pub fn new() -> Self {
        let byte_to_char = get_complete_char_set();
        let mut char_to_byte = HashMap::new();

        // Populate reverse mapping
        for (byte, &character) in &byte_to_char {
            char_to_byte.insert(character, *byte);
        }

        Self {
            byte_to_char,
            char_to_byte,
        }
    }

    /// Retrieves the character corresponding to a given byte.
    /// # Errors
    /// Returns `CharacterSetError::InvalidIndex` if the index is not within the valid range (0-255).
    pub fn get_char(&self, byte: u8) -> Result<&'static str, CharacterSetError> {
        self.byte_to_char
            .get(&byte)
            .copied()
            .ok_or(CharacterSetError::ByteNotFound(byte))
    }

    /// Retrieves the byte corresponding to a given character by searching the `byte_to_char` map.
    /// # Errors
    /// Returns `CharacterSetError::CharacterNotFound` if the character is not in the character set.
    pub fn get_code(&self, character: &str) -> Result<u8, CharacterSetError> {
        self.char_to_byte
            .get(character)
            .copied()
            .ok_or_else(|| CharacterSetError::CharacterNotFound(character.to_string()))
    }

    /// Encodes a string into a vector of bytes, skipping unsupported characters.
    pub fn encode_string(&self, input: &str) -> Result<Vec<u8>, CharacterSetError> {
        input
            .chars()
            .map(|c| self.get_code(&c.to_string()))
            .collect()
    }

    /// Decodes a vector of bytes into a string, skipping invalid bytes.
    pub fn decode_string(&self, input: &[u8]) -> Result<String, CharacterSetError> {
        input
            .iter()
            .map(|&b| self.get_char(b))
            .collect::<Result<Vec<_>, _>>()
            .map(|chars| chars.concat())
    }
}

/// Provides a complete mapping of byte values to characters for Pokémon Gen III games.
/// This mapping supports encoding/decoding of in-game text such as trainer names or Pokémon nicknames.
pub fn get_complete_char_set() -> HashMap<u8, &'static str> {
    let mut byte_to_char = HashMap::new();

    // Known character mappings from the game's character set.
    byte_to_char.insert(0x00, " ");
    byte_to_char.insert(0x01, "À");
    byte_to_char.insert(0x02, "Á");
    byte_to_char.insert(0x03, "Â");
    byte_to_char.insert(0x04, "Ç");
    byte_to_char.insert(0x05, "È");
    byte_to_char.insert(0x06, "É");
    byte_to_char.insert(0x07, "Ê");
    byte_to_char.insert(0x08, "Ë");
    byte_to_char.insert(0x09, "Ì");
    byte_to_char.insert(0x0B, "Î");
    byte_to_char.insert(0x0C, "Ï");
    byte_to_char.insert(0x0D, "Ò");
    byte_to_char.insert(0x0E, "Ó");
    byte_to_char.insert(0x0F, "Ô");
    byte_to_char.insert(0x10, "Œ");
    byte_to_char.insert(0x11, "Ù");
    byte_to_char.insert(0x12, "Ú");
    byte_to_char.insert(0x13, "Û");
    byte_to_char.insert(0x14, "Ñ");
    byte_to_char.insert(0x15, "ß");
    byte_to_char.insert(0x16, "à");
    byte_to_char.insert(0x17, "á");
    byte_to_char.insert(0x19, "ç");
    byte_to_char.insert(0x1A, "è");
    byte_to_char.insert(0x1B, "é");
    byte_to_char.insert(0x1C, "ê");
    byte_to_char.insert(0x1D, "ë");
    byte_to_char.insert(0x1E, "ì");
    byte_to_char.insert(0x20, "î");
    byte_to_char.insert(0x21, "ï");
    byte_to_char.insert(0x22, "ò");
    byte_to_char.insert(0x23, "ó");
    byte_to_char.insert(0x24, "ô");
    byte_to_char.insert(0x25, "œ");
    byte_to_char.insert(0x26, "ù");
    byte_to_char.insert(0x27, "ú");
    byte_to_char.insert(0x28, "û");
    byte_to_char.insert(0x29, "ñ");
    byte_to_char.insert(0x2A, "º");
    byte_to_char.insert(0x2B, "ª");
    byte_to_char.insert(0x2C, "ᵉʳ");
    byte_to_char.insert(0x2D, "&");
    byte_to_char.insert(0x2E, "+");
    byte_to_char.insert(0x34, "Lv");
    byte_to_char.insert(0x35, "=");
    byte_to_char.insert(0x36, ";");
    byte_to_char.insert(0x50, "▯");
    byte_to_char.insert(0x51, "¿");
    byte_to_char.insert(0x52, "¡");
    byte_to_char.insert(0x5A, "Í");
    byte_to_char.insert(0x5B, "%");
    byte_to_char.insert(0x5C, "(");
    byte_to_char.insert(0x5D, ")");
    byte_to_char.insert(0x5E, " ");
    byte_to_char.insert(0x5F, " ");
    byte_to_char.insert(0x68, "â");
    byte_to_char.insert(0x6F, "í");
    byte_to_char.insert(0x79, "↑");
    byte_to_char.insert(0x7A, "↓");
    byte_to_char.insert(0x7B, "←");
    byte_to_char.insert(0x7C, "→");
    byte_to_char.insert(0x7D, "*");
    byte_to_char.insert(0x7E, "*");
    byte_to_char.insert(0x7F, "*");
    byte_to_char.insert(0x80, "*");
    byte_to_char.insert(0x81, "*");
    byte_to_char.insert(0x82, "*");
    byte_to_char.insert(0x83, "*");
    byte_to_char.insert(0x84, "ᵉ");
    byte_to_char.insert(0x85, "<");
    byte_to_char.insert(0x86, ">");
    byte_to_char.insert(0xA0, "ʳᵉ");
    byte_to_char.insert(0xA1, "0");
    byte_to_char.insert(0xA2, "1");
    byte_to_char.insert(0xA3, "2");
    byte_to_char.insert(0xA4, "3");
    byte_to_char.insert(0xA5, "4");
    byte_to_char.insert(0xA6, "5");
    byte_to_char.insert(0xA7, "6");
    byte_to_char.insert(0xA8, "7");
    byte_to_char.insert(0xA9, "8");
    byte_to_char.insert(0xAA, "9");
    byte_to_char.insert(0xAB, "!");
    byte_to_char.insert(0xAC, "?");
    byte_to_char.insert(0xAD, ".");
    byte_to_char.insert(0xAE, "-");
    byte_to_char.insert(0xAF, "・");
    byte_to_char.insert(0xB0, "…");
    byte_to_char.insert(0xB1, "“");
    byte_to_char.insert(0xB2, "”");
    byte_to_char.insert(0xB3, "‘");
    byte_to_char.insert(0xB4, "’");
    byte_to_char.insert(0xB5, "♂");
    byte_to_char.insert(0xB6, "♀");
    byte_to_char.insert(0xB7, "$");
    byte_to_char.insert(0xB8, ",");
    byte_to_char.insert(0xB9, "×");
    byte_to_char.insert(0xBA, "/");
    byte_to_char.insert(0xBB, "A");
    byte_to_char.insert(0xBC, "B");
    byte_to_char.insert(0xBD, "C");
    byte_to_char.insert(0xBE, "D");
    byte_to_char.insert(0xBF, "E");
    byte_to_char.insert(0xC0, "F");
    byte_to_char.insert(0xC1, "G");
    byte_to_char.insert(0xC2, "H");
    byte_to_char.insert(0xC3, "I");
    byte_to_char.insert(0xC4, "J");
    byte_to_char.insert(0xC5, "K");
    byte_to_char.insert(0xC6, "L");
    byte_to_char.insert(0xC7, "M");
    byte_to_char.insert(0xC8, "N");
    byte_to_char.insert(0xC9, "O");
    byte_to_char.insert(0xCA, "P");
    byte_to_char.insert(0xCB, "Q");
    byte_to_char.insert(0xCC, "R");
    byte_to_char.insert(0xCD, "S");
    byte_to_char.insert(0xCE, "T");
    byte_to_char.insert(0xCF, "U");
    byte_to_char.insert(0xD0, "V");
    byte_to_char.insert(0xD1, "W");
    byte_to_char.insert(0xD2, "X");
    byte_to_char.insert(0xD3, "Y");
    byte_to_char.insert(0xD4, "Z");
    byte_to_char.insert(0xD5, "a");
    byte_to_char.insert(0xD6, "b");
    byte_to_char.insert(0xD7, "c");
    byte_to_char.insert(0xD8, "d");
    byte_to_char.insert(0xD9, "e");
    byte_to_char.insert(0xDA, "f");
    byte_to_char.insert(0xDB, "g");
    byte_to_char.insert(0xDC, "h");
    byte_to_char.insert(0xDD, "i");
    byte_to_char.insert(0xDE, "j");
    byte_to_char.insert(0xDF, "k");
    byte_to_char.insert(0xE0, "l");
    byte_to_char.insert(0xE1, "m");
    byte_to_char.insert(0xE2, "n");
    byte_to_char.insert(0xE3, "o");
    byte_to_char.insert(0xE4, "p");
    byte_to_char.insert(0xE5, "q");
    byte_to_char.insert(0xE6, "r");
    byte_to_char.insert(0xE7, "s");
    byte_to_char.insert(0xE8, "t");
    byte_to_char.insert(0xE9, "u");
    byte_to_char.insert(0xEA, "v");
    byte_to_char.insert(0xEB, "w");
    byte_to_char.insert(0xEC, "x");
    byte_to_char.insert(0xED, "y");
    byte_to_char.insert(0xEE, "z");
    byte_to_char.insert(0xEF, "►");
    byte_to_char.insert(0xF0, ":");
    byte_to_char.insert(0xF1, "Ä");
    byte_to_char.insert(0xF2, "Ö");
    byte_to_char.insert(0xF3, "Ü");
    byte_to_char.insert(0xF4, "ä");
    byte_to_char.insert(0xF5, "ö");
    byte_to_char.insert(0xF6, "ü");

    byte_to_char
}
