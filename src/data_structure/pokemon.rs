use byteorder::{ByteOrder, LittleEndian};
use csv::Reader;
use lazy_static::lazy_static;
use serde_json::Value;
use std::fmt;

use crate::data_structure::character_set::get_char;
use crate::data_structure::save_data::TrainerID;

const SPECIES: [u16; 136] = [
    412, 277, 278, 279, 280, 281, 282, 283, 284, 285, 286, 287, 288, 289, 290, 291, 292, 293, 294,
    295, 296, 297, 298, 299, 300, 304, 305, 309, 310, 392, 393, 394, 311, 312, 306, 307, 364, 365,
    366, 301, 302, 303, 370, 371, 372, 335, 336, 350, 320, 315, 316, 322, 355, 382, 383, 384, 356,
    357, 337, 338, 353, 354, 386, 387, 363, 367, 368, 330, 331, 313, 314, 339, 340, 321, 351, 352,
    308, 332, 333, 334, 344, 345, 358, 359, 380, 379, 348, 349, 323, 324, 326, 327, 318, 319, 388,
    389, 390, 391, 328, 329, 385, 317, 377, 378, 361, 362, 369, 411, 376, 360, 346, 347, 341, 342,
    343, 373, 374, 375, 381, 325, 395, 396, 397, 398, 399, 400, 401, 402, 403, 407, 408, 404, 405,
    406, 409, 410,
];

const GENDER_THRESHOLD: [(u32, &str); 8] = [
    (255, "Genderless"),
    (254, "0:100"),
    (225, "12.5:87.5"),
    (191, "75:25"),
    (127, "50:50"),
    (63, "25:75"),
    (31, "87.5:12.5"),
    (0, "100:0"),
];

//  Erratic[0]  Fast[1] M Fast[2]   M Slow[3]   Slow[4] Fluctuating[5]  Level[6]
const EXPERIENCE_TABLE: [[u32; 7]; 100] = [
    [0, 0, 0, 0, 0, 0, 1],
    [15, 6, 8, 9, 10, 4, 2],
    [52, 21, 27, 57, 33, 13, 3],
    [122, 51, 64, 96, 80, 32, 4],
    [237, 100, 125, 135, 156, 65, 5],
    [406, 172, 216, 179, 270, 112, 6],
    [637, 274, 343, 236, 428, 178, 7],
    [942, 409, 512, 314, 640, 276, 8],
    [1326, 583, 729, 419, 911, 393, 9],
    [1800, 800, 1000, 560, 1250, 540, 10],
    [2369, 1064, 1331, 742, 1663, 745, 11],
    [3041, 1382, 1728, 973, 2160, 967, 12],
    [3822, 1757, 2197, 1261, 2746, 1230, 13],
    [4719, 2195, 2744, 1612, 3430, 1591, 14],
    [5737, 2700, 3375, 2035, 4218, 1957, 15],
    [6881, 3276, 4096, 2535, 5120, 2457, 16],
    [8155, 3930, 4913, 3120, 6141, 3046, 17],
    [9564, 4665, 5832, 3798, 7290, 3732, 18],
    [11111, 5487, 6859, 4575, 8573, 4526, 19],
    [12800, 6400, 8000, 5460, 10000, 5440, 20],
    [14632, 7408, 9261, 6458, 11576, 6482, 21],
    [16610, 8518, 10648, 7577, 13310, 7666, 22],
    [18737, 9733, 12167, 8825, 15208, 9003, 23],
    [21012, 11059, 13824, 10208, 17280, 10506, 24],
    [23437, 12500, 15625, 11735, 19531, 12187, 25],
    [26012, 14060, 17576, 13411, 21970, 14060, 26],
    [28737, 15746, 19683, 15244, 24603, 16140, 27],
    [31610, 17561, 21952, 17242, 27440, 18439, 28],
    [34632, 19511, 24389, 19411, 30486, 20974, 29],
    [37800, 21600, 27000, 21760, 33750, 23760, 30],
    [41111, 23832, 29791, 24294, 37238, 26811, 31],
    [44564, 26214, 32768, 27021, 40960, 30146, 32],
    [48155, 28749, 35937, 29949, 44921, 33780, 33],
    [51881, 31443, 39304, 33084, 49130, 37731, 34],
    [55737, 34300, 42875, 36435, 53593, 42017, 35],
    [59719, 37324, 46656, 40007, 58320, 46656, 36],
    [63822, 40522, 50653, 43808, 63316, 50653, 37],
    [68041, 43897, 54872, 47846, 68590, 55969, 38],
    [72369, 47455, 59319, 52127, 74148, 60505, 39],
    [76800, 51200, 64000, 56660, 80000, 66560, 40],
    [81326, 55136, 68921, 61450, 86151, 71677, 41],
    [85942, 59270, 74088, 66505, 92610, 78533, 42],
    [90637, 63605, 79507, 71833, 99383, 84277, 43],
    [95406, 68147, 85184, 77440, 106480, 91998, 44],
    [100237, 72900, 91125, 83335, 113906, 98415, 45],
    [105122, 77868, 97336, 89523, 121670, 107069, 46],
    [110052, 83058, 103823, 96012, 129778, 114205, 47],
    [115015, 88473, 110592, 102810, 138240, 123863, 48],
    [120001, 94119, 117649, 109923, 147061, 131766, 49],
    [125000, 100000, 125000, 117360, 156250, 142500, 50],
    [131324, 106120, 132651, 125126, 165813, 151222, 51],
    [137795, 112486, 140608, 133229, 175760, 163105, 52],
    [144410, 119101, 148877, 141677, 186096, 172697, 53],
    [151165, 125971, 157464, 150476, 196830, 185807, 54],
    [158056, 133100, 166375, 159635, 207968, 196322, 55],
    [165079, 140492, 175616, 169159, 219520, 210739, 56],
    [172229, 148154, 185193, 179056, 231491, 222231, 57],
    [179503, 156089, 195112, 189334, 243890, 238036, 58],
    [186894, 164303, 205379, 199999, 256723, 250562, 59],
    [194400, 172800, 216000, 211060, 270000, 267840, 60],
    [202013, 181584, 226981, 222522, 283726, 281456, 61],
    [209728, 190662, 238328, 234393, 297910, 300293, 62],
    [217540, 200037, 250047, 246681, 312558, 315059, 63],
    [225443, 209715, 262144, 259392, 327680, 335544, 64],
    [233431, 219700, 274625, 272535, 343281, 351520, 65],
    [241496, 229996, 287496, 286115, 359370, 373744, 66],
    [249633, 240610, 300763, 300140, 375953, 390991, 67],
    [257834, 251545, 314432, 314618, 393040, 415050, 68],
    [267406, 262807, 328509, 329555, 410636, 433631, 69],
    [276458, 274400, 343000, 344960, 428750, 459620, 70],
    [286328, 286328, 357911, 360838, 447388, 479600, 71],
    [296358, 298598, 373248, 377197, 466560, 507617, 72],
    [305767, 311213, 389017, 394045, 486271, 529063, 73],
    [316074, 324179, 405224, 411388, 506530, 559209, 74],
    [326531, 337500, 421875, 429235, 527343, 582187, 75],
    [336255, 351180, 438976, 447591, 548720, 614566, 76],
    [346965, 365226, 456533, 466464, 570666, 639146, 77],
    [357812, 379641, 474552, 485862, 593190, 673863, 78],
    [367807, 394431, 493039, 505791, 616298, 700115, 79],
    [378880, 409600, 512000, 526260, 640000, 737280, 80],
    [390077, 425152, 531441, 547274, 664301, 765275, 81],
    [400293, 441094, 551368, 568841, 689210, 804997, 82],
    [411686, 457429, 571787, 590969, 714733, 834809, 83],
    [423190, 474163, 592704, 613664, 740880, 877201, 84],
    [433572, 491300, 614125, 636935, 767656, 908905, 85],
    [445239, 508844, 636056, 660787, 795070, 954084, 86],
    [457001, 526802, 658503, 685228, 823128, 987754, 87],
    [467489, 545177, 681472, 710266, 851840, 1035837, 88],
    [479378, 563975, 704969, 735907, 881211, 1071552, 89],
    [491346, 583200, 729000, 762160, 911250, 1122660, 90],
    [501878, 602856, 753571, 789030, 941963, 1160499, 91],
    [513934, 622950, 778688, 816525, 973360, 1214753, 92],
    [526049, 643485, 804357, 844653, 1005446, 1254796, 93],
    [536557, 664467, 830584, 873420, 1038230, 1312322, 94],
    [548720, 685900, 857375, 902835, 1071718, 1354652, 95],
    [560922, 707788, 884736, 932903, 1105920, 1415577, 96],
    [571333, 730138, 912673, 963632, 1140841, 1460276, 97],
    [583539, 752953, 941192, 995030, 1176490, 1524731, 98],
    [591882, 776239, 970299, 1027103, 1212873, 1571884, 99],
    [600000, 800000, 1000000, 1059860, 1250000, 1640000, 100],
];

const NATURE: [&str; 25] = [
    "Hardy", "Lonely", "Brave", "Adamant", "Naughty", "Bold", "Docile", "Relaxed", "Impish", "Lax",
    "Timid", "Hasty", "Serious", "Jolly", "Naive", "Modest", "Mild", "Quiet", "Bashful", "Rash",
    "Calm", "Gentle", "Sassy", "Careful", "Quirky",
];

const NATURE_MODIFIER: [[f32; 5]; 25] = [
    [1.0, 1.0, 1.0, 1.0, 1.0],
    [1.1, 0.9, 1.0, 1.0, 1.0],
    [1.1, 1.0, 0.9, 1.0, 1.0],
    [1.1, 1.0, 1.0, 0.9, 1.0],
    [1.1, 1.0, 1.0, 1.0, 0.9],
    [0.9, 1.1, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0, 1.0],
    [1.0, 1.1, 0.9, 1.0, 1.0],
    [1.0, 1.1, 1.0, 0.9, 1.0],
    [1.0, 1.1, 1.0, 1.0, 0.9],
    [0.9, 1.0, 1.1, 1.0, 1.0],
    [1.0, 0.9, 1.1, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.1, 0.9, 1.0],
    [1.0, 1.0, 1.1, 1.0, 0.9],
    [0.9, 1.0, 1.0, 1.1, 1.0],
    [1.0, 0.9, 1.0, 1.1, 1.0],
    [1.0, 1.0, 0.9, 1.1, 1.0],
    [1.0, 1.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.1, 0.9],
    [0.9, 1.0, 1.0, 1.0, 1.1],
    [1.0, 0.9, 1.0, 1.0, 1.1],
    [1.0, 1.0, 0.9, 1.0, 1.1],
    [1.0, 1.0, 1.0, 0.9, 1.1],
    [1.0, 1.0, 1.0, 1.0, 1.0],
];

const POKEDEX_BYTES: &[u8] = include_bytes!("../../pokedex.csv");
const POKEDEX_JSON_BYTES: &[u8] = include_bytes!("../../pokedex.json");
const MOVES_BYTES: &[u8] = include_bytes!("../../moves.json");
const ITEMS_BYTES: &[u8] = include_bytes!("../../items.json");
const ITEMS_G3_BYTES: &[u8] = include_bytes!("../../items.csv");

lazy_static! {
    static ref POKEDEX: Vec<csv::StringRecord> = Reader::from_reader(POKEDEX_BYTES)
        .records()
        .map(|record| record.unwrap())
        .collect();
    static ref POKEDEX_JSON: Vec<Value> = serde_json::from_reader(POKEDEX_JSON_BYTES).unwrap();
    static ref MOVES: Vec<Value> = serde_json::from_reader(MOVES_BYTES).unwrap();
    static ref ITEMS: Vec<Value> = serde_json::from_reader(ITEMS_BYTES).unwrap();
    static ref ITEMS_G3: Vec<csv::StringRecord> = Reader::from_reader(ITEMS_G3_BYTES)
        .records()
        .map(|record| record.unwrap())
        .collect();
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Pokemon {
    offset: usize,
    personality_value: [u8; 4],
    ot_id: [u8; 4],
    nickname: [u8; 10],
    language: [u8; 1],
    misc_flags: [u8; 1],
    ot_name: [u8; 7],
    markings: [u8; 1],
    checksum: [u8; 2],
    _padding: [u8; 2],
    pokemon_data: PokemonData,
}

impl Pokemon {
    pub fn new(offset: usize, buffer: &[u8]) -> Self {
        let pokemon_data = &buffer[0x20..0x50];

        // unencrypt the pokemon data substructure
        // first we obtain the ecryption key by XORing the entire original trainer id with the pokemon personality value
        let ecryption_key = LittleEndian::read_u32(&buffer[0x04..0x08])
            ^ LittleEndian::read_u32(&buffer[0x00..0x04]);

        let mut data: [u8; 48] = [0; 48];
        pokemon_data_encryption(ecryption_key, pokemon_data, &mut data);

        // This section is stored in a special and encrypted format.
        // The order of the structures is determined by the personality value of the Pokémon modulo 24
        let personality_value_modulo = LittleEndian::read_u32(&buffer[0x00..0x04]) % 24;
        let mut pokemon_data = PokemonData::new(data);

        order_data_substructure(personality_value_modulo, &mut pokemon_data);

        let mut personality_value = [0; 4];
        let mut ot_id = [0; 4];
        let mut nickname = [0; 10];
        let mut language = [0];
        let mut misc_flags = [0];
        let mut ot_name = [0; 7];
        let mut markings = [0];
        let mut checksum = [0; 2];
        let mut _padding = [0; 2];

        personality_value.copy_from_slice(&buffer[0x00..0x04]);
        ot_id.copy_from_slice(&buffer[0x04..0x08]);
        nickname.copy_from_slice(&buffer[0x08..0x12]);
        language.copy_from_slice(&buffer[0x12..0x13]);
        misc_flags.copy_from_slice(&buffer[0x13..0x14]);
        ot_name.copy_from_slice(&buffer[0x14..0x1B]);
        markings.copy_from_slice(&buffer[0x1B..0x1C]);
        checksum.copy_from_slice(&buffer[0x1C..0x1E]);
        _padding.copy_from_slice(&buffer[0x1E..0x20]);

        Pokemon {
            offset,
            personality_value,
            ot_id,
            nickname,
            language,
            misc_flags,
            ot_name,
            markings,
            checksum,
            _padding,
            pokemon_data,
        }
    }

    pub fn ofsset(&self) -> usize {
        self.offset
    }

    pub fn ot_id(&self) -> TrainerID {
        self.ot_id.into()
    }

    pub fn nickname(&self) -> String {
        let nickname = &self
            .nickname
            .iter()
            .map(|c| get_char(*c as usize))
            .collect::<Vec<&str>>();

        let nickname = nickname.join("");
        let nickname = nickname.split(' ').next().unwrap();

        nickname.to_string()
    }

    pub fn language(&self) -> Language {
        self.language.into()
    }

    pub fn ot_name(&self) -> String {
        let ot_name = &self
            .ot_name
            .iter()
            .map(|c| get_char(*c as usize))
            .collect::<Vec<&str>>();

        let ot_name = ot_name.join("");
        let ot_name = ot_name.split(' ').next().unwrap();

        ot_name.to_string()
    }

    pub fn checksum(&self) -> u16 {
        LittleEndian::read_u16(&self.checksum)
    }

    pub fn species(&self) -> String {
        let dex_num = self.nat_dex_number();
        let index = dex_num.saturating_sub(1);

        if dex_num != 0 {
            POKEDEX[index as usize].get(1).unwrap().to_string()
        } else {
            "".to_string()
        }
    }

    pub fn nat_dex_number(&self) -> u16 {
        let species = self.species_id();

        if species == 412 {
            return 0;
        }
        if species >= 277 {
            return (SPECIES.iter().position(|&x| x == species).unwrap() + 251)
                .try_into()
                .unwrap();
        }

        species
    }

    pub fn experience(&self) -> u32 {
        let offset = self.pokemon_data.growth_offset;
        LittleEndian::read_u32(&self.pokemon_data.data[offset + 4..offset + 8])
    }

    pub fn gender(&self) -> Gender {
        let p = self.personality_value();
        let pg = p % 256;

        if p == 0 {
            Gender::None
        } else {
            let dex_num = self.nat_dex_number();

            let threshold = gender_threshold(dex_num.into());

            if threshold == 255 {
                Gender::None
            } else if pg >= threshold {
                Gender::M
            } else {
                Gender::F
            }
        }
    }

    pub fn level(&self) -> u8 {
        let mut level: u32 = 0;

        if self.is_empty() {
            return level as u8;
        }

        let index = self.nat_dex_number();

        let pokemon = POKEDEX
            .iter()
            .find(|mon| mon.get(0).unwrap().replace('#', "").parse::<u16>().unwrap() == index);

        let growth = pokemon.unwrap().get(11);

        let growth_index = growth_index(growth.unwrap());

        let experience = self.experience();

        let mut iter = EXPERIENCE_TABLE.iter().peekable();

        while let Some(current) = iter.next() {
            if let Some(peek) = iter.peek() {
                if current[growth_index] <= experience && experience < peek[growth_index] {
                    level = current[6];
                    break;
                }
            } else {
                level = current[6];
            }
        }

        level as u8
    }

    pub fn typing(&self) -> Option<(String, Option<String>)> {
        if self.is_empty() {
            return None;
        }

        let index = self.nat_dex_number();

        let pokemon = POKEDEX
            .iter()
            .find(|mon| mon.get(0).unwrap().replace('#', "").parse::<u16>().unwrap() == index);

        let type1 = pokemon.unwrap().get(2);
        let type2 = pokemon.unwrap().get(3);

        if type2 == Some("Nan") {
            Some((type1.unwrap().to_string(), None))
        } else {
            Some((type1.unwrap().to_string(), Some(type2.unwrap().to_string())))
        }
    }

    pub fn ability(&self) -> String {
        let index = self.nat_dex_number().saturating_sub(1) as usize;
        let ability_index = self.ability_index();

        if self.is_empty() {
            return String::from("");
        }

        POKEDEX_JSON[index]["profile"]["ability"][ability_index][0]
            .as_str()
            .unwrap()
            .to_string()
    }

    pub fn moves(&self) -> Vec<(String, String, u8, u8)> {
        let offset = self.pokemon_data.attacks_offset;
        let move1_index =
            LittleEndian::read_u16(&self.pokemon_data.data[offset..offset + 2]) as usize;
        let move2_index =
            LittleEndian::read_u16(&self.pokemon_data.data[offset + 2..offset + 4]) as usize;
        let move3_index =
            LittleEndian::read_u16(&self.pokemon_data.data[offset + 4..offset + 6]) as usize;
        let move4_index =
            LittleEndian::read_u16(&self.pokemon_data.data[offset + 6..offset + 8]) as usize;

        let pp1 = &self.pokemon_data.data[offset + 8..offset + 9];
        let pp2 = &self.pokemon_data.data[offset + 9..offset + 10];
        let pp3 = &self.pokemon_data.data[offset + 10..offset + 11];
        let pp4 = &self.pokemon_data.data[offset + 11..offset + 12];

        let mut moves: Vec<(String, String, u8, u8)> = vec![];

        let move1 = MOVES.iter().find(|&m| m["id"] == move1_index);
        let move2 = MOVES.iter().find(|&m| m["id"] == move2_index);
        let move3 = MOVES.iter().find(|&m| m["id"] == move3_index);
        let move4 = MOVES.iter().find(|&m| m["id"] == move4_index);

        if let Some(p_move) = move1 {
            moves.push((
                p_move["type"].as_str().unwrap().to_string(),
                p_move["ename"].as_str().unwrap().to_string(),
                p_move["pp"].as_u64().unwrap() as u8,
                pp1[0],
            ));
        }

        if let Some(p_move) = move2 {
            moves.push((
                p_move["type"].as_str().unwrap().to_string(),
                p_move["ename"].as_str().unwrap().to_string(),
                p_move["pp"].as_u64().unwrap() as u8,
                pp2[0],
            ));
        }

        if let Some(p_move) = move3 {
            moves.push((
                p_move["type"].as_str().unwrap().to_string(),
                p_move["ename"].as_str().unwrap().to_string(),
                p_move["pp"].as_u64().unwrap() as u8,
                pp3[0],
            ));
        }

        if let Some(p_move) = move4 {
            moves.push((
                p_move["type"].as_str().unwrap().to_string(),
                p_move["ename"].as_str().unwrap().to_string(),
                p_move["pp"].as_u64().unwrap() as u8,
                pp4[0],
            ));
        }

        moves
    }

    pub fn held_item(&self) -> String {
        let offset = self.pokemon_data.growth_offset;
        let held_item_index =
            LittleEndian::read_u16(&self.pokemon_data.data[offset + 2..offset + 4]) as usize;

        if held_item_index == 0 {
            return String::from("-");
        }

        let item_name = ITEMS_G3[held_item_index]
            .get(3)
            .unwrap()
            .to_string()
            .replace('*', "");

        item_name
    }

    pub fn pokeball_caught(&self) -> usize {
        let offset = self.pokemon_data.miscellaneous_offset;
        let origins_info = &self.pokemon_data.data[offset + 2..offset + 4];
        // mask to get the bits 11 - 14
        //0x7800 = 0b0111100000000000
        const BITS_MASK: u16 = 0x7800;

        ((LittleEndian::read_u16(origins_info) & BITS_MASK) >> 11) as usize
    }

    pub fn pokerus_status(&self) -> Pokerus {
        let offset = self.pokemon_data.miscellaneous_offset;
        let pokerus = &self.pokemon_data.data[offset..offset + 1];
        //strain    = 0xF0  = 0b11110000
        //days left = 0xF   = 0b00001111
        const DAYS_MASK: u8 = 0xF;
        const STRAIN_MASK: u8 = 0xF0;

        let strain: u8 = (pokerus[0] & STRAIN_MASK) >> 4;
        let days: u8 = pokerus[0] & DAYS_MASK;

        if strain > 0 && days == 0 {
            Pokerus::Cured
        } else if strain > 0 && days > 0 {
            Pokerus::Infected
        } else {
            Pokerus::None
        }
    }

    pub fn nature(&self) -> String {
        if !self.is_empty() {
            let p = self.nature_index();

            NATURE[p].to_string()
        } else {
            String::from("")
        }
    }

    pub fn stats(&self) {
        let index = self.nat_dex_number().saturating_sub(1) as usize;
        let nature_index = self.nature_index();

        let base_stats = &POKEDEX_JSON[index]["base"];
        let ev_offset = self.pokemon_data.ev_offset;

        println!(
            "{:?}",
            self.pokemon_data.data[ev_offset..ev_offset + 1][0] as u16
        );

        let iv_offset = self.pokemon_data.miscellaneous_offset;

        let ivs = LittleEndian::read_u32(&self.pokemon_data.data[iv_offset + 4..iv_offset + 8]);

        let stats = Stats {
            // Base
            hp: base_stats["HP"].as_u64().unwrap() as u16,
            attack: base_stats["Attack"].as_u64().unwrap() as u16,
            defense: base_stats["Defense"].as_u64().unwrap() as u16,
            sp_attack: base_stats["Sp. Attack"].as_u64().unwrap() as u16,
            sp_defense: base_stats["Sp. Defense"].as_u64().unwrap() as u16,
            speed: base_stats["Speed"].as_u64().unwrap() as u16,
            // Effort Values
            hp_ev: self.pokemon_data.data[ev_offset..ev_offset + 1][0] as u16,
            attack_ev: self.pokemon_data.data[ev_offset + 1..ev_offset + 2][0] as u16,
            defense_ev: self.pokemon_data.data[ev_offset + 2..ev_offset + 3][0] as u16,
            sp_attack_ev: self.pokemon_data.data[ev_offset + 4..ev_offset + 5][0] as u16,
            sp_defense_ev: self.pokemon_data.data[ev_offset + 5..ev_offset + 6][0] as u16,
            speed_ev: self.pokemon_data.data[ev_offset + 3..ev_offset + 4][0] as u16,
            // Individual Values
            // HP           0xF         = 0b00000000000000000000000000001111
            hp_iv: (ivs & 0xF) as u16,
            // Attack       0xF0        = 0b00000000000000000000000011110000
            attack_iv: (ivs & 0xF0) as u16,
            // Defense      0xF00       = 0b00000000000000000000111100000000
            defense_iv: (ivs & 0xF00) as u16,
            // Sp Attack    0xF0000     = 0b00000000000011110000000000000000
            sp_attack_iv: (ivs & 0xF0000) as u16,
            // Sp Defense   0xF00000    = 0b00000000111100000000000000000000
            sp_defense_iv: (ivs & 0xF00000) as u16,
            // Speed        0xF000      = 0b00000000000000001111000000000000
            speed_iv: (ivs & 0xF000) as u16,
            // Nature Modifiers
            n_mod: NATURE_MODIFIER[nature_index],
        };

        println!("{:?}", stats);
    }

    pub fn is_egg(&self) -> bool {
        let offset = self.pokemon_data.miscellaneous_offset;
        let iv_egg_ability = &self.pokemon_data.data[offset + 4..offset + 8];
        // mask to get the 30 bit
        // 0x40000000 = 0b01000000000000000000000000000000
        const LOW_1_BITS_MASK: u32 = 0x40000000;

        let bit_value = ((LittleEndian::read_u32(iv_egg_ability) & LOW_1_BITS_MASK) >> 30) as usize;

        if self.is_empty() || bit_value == 0 {
            return false;
        }

        true
    }

    pub fn is_empty(&self) -> bool {
        if self.personality_value.is_empty() || self.personality_value() == 0 {
            return true;
        }

        false
    }

    fn nature_index(&self) -> usize {
        (self.personality_value() % 25) as usize
    }

    fn personality_value(&self) -> u32 {
        LittleEndian::read_u32(&self.personality_value)
    }

    fn species_id(&self) -> u16 {
        let offset = self.pokemon_data.growth_offset;
        LittleEndian::read_u16(&self.pokemon_data.data[offset..offset + 2])
    }

    fn ability_index(&self) -> usize {
        let offset = self.pokemon_data.miscellaneous_offset;
        let iv_egg_ability = &self.pokemon_data.data[offset + 4..offset + 8];
        // mask to get the 31 bit
        //0x7fffffff = 0b01111111111111111111111111111111
        const LOW_1_BITS_MASK: u32 = 0x7fffffff;
        // mask out the ability bit and shift it to the right
        ((LittleEndian::read_u32(iv_egg_ability) & LOW_1_BITS_MASK) >> 31) as usize
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PokemonData {
    data: [u8; 48],
    offset: usize,
    growth_offset: usize,
    attacks_offset: usize,
    ev_offset: usize,
    miscellaneous_offset: usize,
}

impl PokemonData {
    fn new(data: [u8; 48]) -> Self {
        PokemonData {
            data,
            ..PokemonData::default()
        }
    }

    fn set_growth_offset(&mut self, growth_offset: usize) -> &mut Self {
        self.growth_offset = growth_offset;
        self
    }

    fn set_attacks_offset(&mut self, attacks_offset: usize) -> &mut Self {
        self.attacks_offset = attacks_offset;
        self
    }

    fn set_ev_offset(&mut self, ev_offset: usize) -> &mut Self {
        self.ev_offset = ev_offset;
        self
    }

    fn set_miscellaneous_offset(&mut self, miscellaneous_offset: usize) -> &mut Self {
        self.miscellaneous_offset = miscellaneous_offset;
        self
    }

    // calculate checksum
    // To validate the checksum given in the encapsulating Pokémon data structure, the entirety of the four unencrypted data substructures must be summed into a 16-bit value
    // We accomplish this by splitting the unencrypted data substructure in chunks of 2 bytes (u16) and summing every chunk
    fn checksum(&self) -> u16 {
        let mut checksum: u16 = 0;
        for chunk in self.data.chunks(2) {
            let (sum, _) = checksum.overflowing_add(LittleEndian::read_u16(chunk));
            checksum = sum;
        }

        checksum
    }
}

impl Default for PokemonData {
    fn default() -> Self {
        PokemonData {
            data: [0; 48],
            offset: 0x20,
            growth_offset: 0,
            attacks_offset: 0,
            ev_offset: 0,
            miscellaneous_offset: 0,
        }
    }
}

#[derive(Debug, Default)]
struct Stats {
    // Base
    hp: u16,
    attack: u16,
    defense: u16,
    sp_attack: u16,
    sp_defense: u16,
    speed: u16,
    // Effort Values
    hp_ev: u16,
    attack_ev: u16,
    defense_ev: u16,
    sp_attack_ev: u16,
    sp_defense_ev: u16,
    speed_ev: u16,
    // Individual Values
    hp_iv: u16,
    attack_iv: u16,
    defense_iv: u16,
    sp_attack_iv: u16,
    sp_defense_iv: u16,
    speed_iv: u16,
    // Nature Modifiers
    n_mod: [f32; 5],
}

#[derive(Debug)]
pub enum Language {
    Japanese,
    English,
    French,
    Italian,
    German,
    Unused,
    Spanish,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Japanese => write!(f, "JAP"),
            Language::English => write!(f, "ENG"),
            Language::French => write!(f, "FRE"),
            Language::Italian => write!(f, "ITA"),
            Language::German => write!(f, "GER"),
            Language::Unused => write!(f, ""),
            Language::Spanish => write!(f, "SPA"),
        }
    }
}

impl From<[u8; 1]> for Language {
    fn from(language: [u8; 1]) -> Self {
        match language {
            [1] => Language::Japanese,
            [2] => Language::English,
            [3] => Language::French,
            [4] => Language::Italian,
            [5] => Language::German,
            [7] => Language::Spanish,
            _ => Language::Unused,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Gender {
    M,
    F,
    #[default]
    None,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Pokerus {
    #[default]
    None,
    Infected,
    Cured,
}

fn order_data_substructure(key: u32, pokemon_data: &mut PokemonData) {
    if key == 0 {
        pokemon_data.set_growth_offset(0);
        pokemon_data.set_attacks_offset(12);
        pokemon_data.set_ev_offset(24);
        pokemon_data.set_miscellaneous_offset(36);
    }
    if key == 1 {
        pokemon_data.set_growth_offset(0);
        pokemon_data.set_attacks_offset(12);
        pokemon_data.set_miscellaneous_offset(24);
        pokemon_data.set_ev_offset(36);
    }
    if key == 2 {
        pokemon_data.set_growth_offset(0);
        pokemon_data.set_ev_offset(12);
        pokemon_data.set_attacks_offset(24);
        pokemon_data.set_miscellaneous_offset(36);
    }
    if key == 3 {
        pokemon_data.set_growth_offset(0);
        pokemon_data.set_ev_offset(12);
        pokemon_data.set_miscellaneous_offset(24);
        pokemon_data.set_attacks_offset(36);
    }
    if key == 4 {
        pokemon_data.set_growth_offset(0);
        pokemon_data.set_miscellaneous_offset(12);
        pokemon_data.set_attacks_offset(24);
        pokemon_data.set_ev_offset(36);
    }
    if key == 5 {
        pokemon_data.set_growth_offset(0);
        pokemon_data.set_miscellaneous_offset(12);
        pokemon_data.set_ev_offset(24);
        pokemon_data.set_attacks_offset(36);
    }
    if key == 6 {
        pokemon_data.set_attacks_offset(0);
        pokemon_data.set_growth_offset(12);
        pokemon_data.set_ev_offset(24);
        pokemon_data.set_miscellaneous_offset(36);
    }
    if key == 7 {
        pokemon_data.set_attacks_offset(0);
        pokemon_data.set_growth_offset(12);
        pokemon_data.set_miscellaneous_offset(24);
        pokemon_data.set_ev_offset(36);
    }
    if key == 8 {
        pokemon_data.set_attacks_offset(0);
        pokemon_data.set_ev_offset(12);
        pokemon_data.set_growth_offset(24);
        pokemon_data.set_miscellaneous_offset(36);
    }
    if key == 9 {
        pokemon_data.set_attacks_offset(0);
        pokemon_data.set_ev_offset(12);
        pokemon_data.set_miscellaneous_offset(24);
        pokemon_data.set_growth_offset(36);
    }
    if key == 10 {
        pokemon_data.set_attacks_offset(0);
        pokemon_data.set_miscellaneous_offset(12);
        pokemon_data.set_growth_offset(24);
        pokemon_data.set_ev_offset(36);
    }
    if key == 11 {
        pokemon_data.set_attacks_offset(0);
        pokemon_data.set_miscellaneous_offset(12);
        pokemon_data.set_ev_offset(24);
        pokemon_data.set_growth_offset(36);
    }
    if key == 12 {
        pokemon_data.set_ev_offset(0);
        pokemon_data.set_growth_offset(12);
        pokemon_data.set_attacks_offset(24);
        pokemon_data.set_miscellaneous_offset(36);
    }
    if key == 13 {
        pokemon_data.set_ev_offset(0);
        pokemon_data.set_growth_offset(12);
        pokemon_data.set_miscellaneous_offset(24);
        pokemon_data.set_attacks_offset(36);
    }
    if key == 14 {
        pokemon_data.set_ev_offset(0);
        pokemon_data.set_attacks_offset(12);
        pokemon_data.set_growth_offset(24);
        pokemon_data.set_miscellaneous_offset(36);
    }
    if key == 15 {
        pokemon_data.set_ev_offset(0);
        pokemon_data.set_attacks_offset(12);
        pokemon_data.set_miscellaneous_offset(24);
        pokemon_data.set_growth_offset(36);
    }
    if key == 16 {
        pokemon_data.set_ev_offset(0);
        pokemon_data.set_miscellaneous_offset(12);
        pokemon_data.set_growth_offset(24);
        pokemon_data.set_attacks_offset(36);
    }
    if key == 17 {
        pokemon_data.set_ev_offset(0);
        pokemon_data.set_miscellaneous_offset(12);
        pokemon_data.set_attacks_offset(24);
        pokemon_data.set_growth_offset(36);
    }
    if key == 18 {
        pokemon_data.set_miscellaneous_offset(0);
        pokemon_data.set_growth_offset(12);
        pokemon_data.set_attacks_offset(24);
        pokemon_data.set_ev_offset(36);
    }
    if key == 19 {
        pokemon_data.set_miscellaneous_offset(0);
        pokemon_data.set_growth_offset(12);
        pokemon_data.set_ev_offset(24);
        pokemon_data.set_attacks_offset(36);
    }
    if key == 20 {
        pokemon_data.set_miscellaneous_offset(0);
        pokemon_data.set_attacks_offset(12);
        pokemon_data.set_growth_offset(24);
        pokemon_data.set_ev_offset(36);
    }
    if key == 21 {
        pokemon_data.set_miscellaneous_offset(0);
        pokemon_data.set_attacks_offset(12);
        pokemon_data.set_ev_offset(24);
        pokemon_data.set_growth_offset(36);
    }
    if key == 22 {
        pokemon_data.set_miscellaneous_offset(0);
        pokemon_data.set_ev_offset(12);
        pokemon_data.set_growth_offset(24);
        pokemon_data.set_attacks_offset(36);
    }
    if key == 23 {
        pokemon_data.set_miscellaneous_offset(0);
        pokemon_data.set_ev_offset(12);
        pokemon_data.set_attacks_offset(24);
        pokemon_data.set_growth_offset(36);
    }
}

fn pokemon_data_encryption(key: u32, data: &[u8], new_data: &mut [u8]) {
    // second we decrypt the data by XORing it, 32 bits (or 4 bytes) at a time with the encryption key
    // We accomplish this by splitting the data substructure in chunks of 4 bytes (u32) and XORing every chunk
    for (i, chunk) in data.chunks(4).enumerate() {
        let i_offest = 4 * i;
        let xored = LittleEndian::read_u32(chunk) ^ key;
        new_data[i_offest..i_offest + 4].copy_from_slice(&xored.to_le_bytes());
    }
}

pub fn transpose_item(name: &str) -> Option<usize> {
    let item = ITEMS.iter().find(|&item| item["name"]["english"] == name);

    item.map(|item| item["id"].as_u64().unwrap() as usize)
}

fn growth_index(growth: &str) -> usize {
    match growth {
        "Erratic" => 0,
        "Fast" => 1,
        "Medium Fast" => 2,
        "Medium Slow" => 3,
        "Slow" => 4,
        "Fluctuating" => 5,
        _ => 7,
    }
}

fn gender_threshold(index: usize) -> u32 {
    let pokemon = POKEDEX.iter().find(|mon| {
        mon.get(0)
            .unwrap()
            .replace('#', "")
            .parse::<usize>()
            .unwrap()
            == index
    });

    let gender_m = pokemon.unwrap().get(12);
    let gender_f = pokemon.unwrap().get(13);

    let gender = format!(
        "{}:{}",
        gender_m.unwrap().replace("% male", ""),
        gender_f.unwrap().replace("% female", "").trim()
    );

    let mut iter = GENDER_THRESHOLD
        .iter()
        .filter(|(_, g)| *g.to_string() == gender);

    let Some((threshold, _)) = iter.next() else {
        return 255;
    };

    *threshold
}
