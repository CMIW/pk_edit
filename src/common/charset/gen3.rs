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

fn get_byte_set() -> HashMap<&'static str, u8> {
    let mut char_to_byte = HashMap::new();
    char_to_byte.insert(" ", 0x00);
    char_to_byte.insert("À", 0x01);
    char_to_byte.insert("Á", 0x02);
    char_to_byte.insert("Â", 0x03);
    char_to_byte.insert("Ç", 0x04);
    char_to_byte.insert("È", 0x05);
    char_to_byte.insert("É", 0x06);
    char_to_byte.insert("Ê", 0x07);
    char_to_byte.insert("Ë", 0x08);
    char_to_byte.insert("Ì", 0x09);
    char_to_byte.insert("Î", 0x0B);
    char_to_byte.insert("Ï", 0x0C);
    char_to_byte.insert("Ò", 0x0D);
    char_to_byte.insert("Ó", 0x0E);
    char_to_byte.insert("Ô", 0x0F);
    char_to_byte.insert("Œ", 0x10);
    char_to_byte.insert("Ù", 0x11);
    char_to_byte.insert("Ú", 0x12);
    char_to_byte.insert("Û", 0x13);
    char_to_byte.insert("Ñ", 0x14);
    char_to_byte.insert("ß", 0x15);
    char_to_byte.insert("à", 0x16);
    char_to_byte.insert("á", 0x17);
    char_to_byte.insert("ç", 0x19);
    char_to_byte.insert("è", 0x1A);
    char_to_byte.insert("é", 0x1B);
    char_to_byte.insert("ê", 0x1C);
    char_to_byte.insert("ë", 0x1D);
    char_to_byte.insert("ì", 0x1E);
    char_to_byte.insert("î", 0x20);
    char_to_byte.insert("ï", 0x21);
    char_to_byte.insert("ò", 0x22);
    char_to_byte.insert("ó", 0x23);
    char_to_byte.insert("ô", 0x24);
    char_to_byte.insert("œ", 0x25);
    char_to_byte.insert("ù", 0x26);
    char_to_byte.insert("ú", 0x27);
    char_to_byte.insert("û", 0x28);
    char_to_byte.insert("ñ", 0x29);
    char_to_byte.insert("º", 0x2A);
    char_to_byte.insert("ª", 0x2B);
    char_to_byte.insert("ᵉʳ", 0x2C);
    char_to_byte.insert("&", 0x2D);
    char_to_byte.insert("+", 0x2E);
    char_to_byte.insert("Lv", 0x34);
    char_to_byte.insert("=", 0x35);
    char_to_byte.insert(";", 0x36);
    char_to_byte.insert("▯", 0x50);
    char_to_byte.insert("¿", 0x51);
    char_to_byte.insert("¡", 0x52);
    char_to_byte.insert("Í", 0x5A);
    char_to_byte.insert("%", 0x5B);
    char_to_byte.insert("(", 0x5C);
    char_to_byte.insert(")", 0x5D);
    char_to_byte.insert(" ", 0x5E);
    char_to_byte.insert(" ", 0x5F);
    char_to_byte.insert("â", 0x68);
    char_to_byte.insert("í", 0x6F);
    char_to_byte.insert("↑", 0x79);
    char_to_byte.insert("↓", 0x7A);
    char_to_byte.insert("←", 0x7B);
    char_to_byte.insert("→", 0x7C);
    char_to_byte.insert("*", 0x7D);
    char_to_byte.insert("*", 0x7E);
    char_to_byte.insert("*", 0x7F);
    char_to_byte.insert("*", 0x80);
    char_to_byte.insert("*", 0x81);
    char_to_byte.insert("*", 0x82);
    char_to_byte.insert("*", 0x83);
    char_to_byte.insert("ᵉ", 0x84);
    char_to_byte.insert("<", 0x85);
    char_to_byte.insert(">", 0x86);
    char_to_byte.insert("ʳᵉ", 0xA0);
    char_to_byte.insert("0", 0xA1);
    char_to_byte.insert("1", 0xA2);
    char_to_byte.insert("2", 0xA3);
    char_to_byte.insert("3", 0xA4);
    char_to_byte.insert("4", 0xA5);
    char_to_byte.insert("5", 0xA6);
    char_to_byte.insert("6", 0xA7);
    char_to_byte.insert("7", 0xA8);
    char_to_byte.insert("8", 0xA9);
    char_to_byte.insert("9", 0xAA);
    char_to_byte.insert("!", 0xAB);
    char_to_byte.insert("?", 0xAC);
    char_to_byte.insert(".", 0xAD);
    char_to_byte.insert("-", 0xAE);
    char_to_byte.insert("・", 0xAF);
    char_to_byte.insert("…", 0xB0);
    char_to_byte.insert("“", 0xB1);
    char_to_byte.insert("”", 0xB2);
    char_to_byte.insert("‘", 0xB3);
    char_to_byte.insert("’", 0xB4);
    char_to_byte.insert("♂", 0xB5);
    char_to_byte.insert("♀", 0xB6);
    char_to_byte.insert("$", 0xB7);
    char_to_byte.insert(",", 0xB8);
    char_to_byte.insert("×", 0xB9);
    char_to_byte.insert("/", 0xBA);
    char_to_byte.insert("A", 0xBB);
    char_to_byte.insert("B", 0xBC);
    char_to_byte.insert("C", 0xBD);
    char_to_byte.insert("D", 0xBE);
    char_to_byte.insert("E", 0xBF);
    char_to_byte.insert("F", 0xC0);
    char_to_byte.insert("G", 0xC1);
    char_to_byte.insert("H", 0xC2);
    char_to_byte.insert("I", 0xC3);
    char_to_byte.insert("J", 0xC4);
    char_to_byte.insert("K", 0xC5);
    char_to_byte.insert("L", 0xC6);
    char_to_byte.insert("M", 0xC7);
    char_to_byte.insert("N", 0xC8);
    char_to_byte.insert("O", 0xC9);
    char_to_byte.insert("P", 0xCA);
    char_to_byte.insert("Q", 0xCB);
    char_to_byte.insert("R", 0xCC);
    char_to_byte.insert("S", 0xCD);
    char_to_byte.insert("T", 0xCE);
    char_to_byte.insert("U", 0xCF);
    char_to_byte.insert("V", 0xD0);
    char_to_byte.insert("W", 0xD1);
    char_to_byte.insert("X", 0xD2);
    char_to_byte.insert("Y", 0xD3);
    char_to_byte.insert("Z", 0xD4);
    char_to_byte.insert("a", 0xD5);
    char_to_byte.insert("b", 0xD6);
    char_to_byte.insert("c", 0xD7);
    char_to_byte.insert("d", 0xD8);
    char_to_byte.insert("e", 0xD9);
    char_to_byte.insert("f", 0xDA);
    char_to_byte.insert("g", 0xDB);
    char_to_byte.insert("h", 0xDC);
    char_to_byte.insert("i", 0xDD);
    char_to_byte.insert("j", 0xDE);
    char_to_byte.insert("k", 0xDF);
    char_to_byte.insert("l", 0xE0);
    char_to_byte.insert("m", 0xE1);
    char_to_byte.insert("n", 0xE2);
    char_to_byte.insert("o", 0xE3);
    char_to_byte.insert("p", 0xE4);
    char_to_byte.insert("q", 0xE5);
    char_to_byte.insert("r", 0xE6);
    char_to_byte.insert("s", 0xE7);
    char_to_byte.insert("t", 0xE8);
    char_to_byte.insert("u", 0xE9);
    char_to_byte.insert("v", 0xEA);
    char_to_byte.insert("w", 0xEB);
    char_to_byte.insert("x", 0xEC);
    char_to_byte.insert("y", 0xED);
    char_to_byte.insert("z", 0xEE);
    char_to_byte.insert("►", 0xEF);
    char_to_byte.insert(":", 0xF0);
    char_to_byte.insert("Ä", 0xF1);
    char_to_byte.insert("Ö", 0xF2);
    char_to_byte.insert("Ü", 0xF3);
    char_to_byte.insert("ä", 0xF4);
    char_to_byte.insert("ö", 0xF5);
    char_to_byte.insert("ü", 0xF6);

    char_to_byte
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
    *get_byte_set().get(s).unwrap()
}
