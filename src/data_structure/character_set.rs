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

pub fn get_char(index: usize) -> &'static str {
    let char_set = get_char_set();

    char_set[index]
}

pub fn get_code(s: &str) -> u8 {
    get_char_set().iter().position(|&c| c == s).unwrap() as u8
}
