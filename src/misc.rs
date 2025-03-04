use rusqlite::{Connection, Result};
use std::fs::File;
use std::io::Write;
use crate::Evolution;

pub const SPECIES: [u16; 136] = [
    412, 277, 278, 279, 280, 281, 282, 283, 284, 285, 286, 287, 288, 289, 290, 291, 292, 293, 294,
    295, 296, 297, 298, 299, 300, 304, 305, 309, 310, 392, 393, 394, 311, 312, 306, 307, 364, 365,
    366, 301, 302, 303, 370, 371, 372, 335, 336, 350, 320, 315, 316, 322, 355, 382, 383, 384, 356,
    357, 337, 338, 353, 354, 386, 387, 363, 367, 368, 330, 331, 313, 314, 339, 340, 321, 351, 352,
    308, 332, 333, 334, 344, 345, 358, 359, 380, 379, 348, 349, 323, 324, 326, 327, 318, 319, 388,
    389, 390, 391, 328, 329, 385, 317, 377, 378, 361, 362, 369, 411, 376, 360, 346, 347, 341, 342,
    343, 373, 374, 375, 381, 325, 395, 396, 397, 398, 399, 400, 401, 402, 403, 407, 408, 404, 405,
    406, 409, 410,
];

pub const GENDER_THRESHOLD: [(u32, &str); 8] = [
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
pub const EXPERIENCE_TABLE: [[u32; 7]; 100] = [
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

pub const NATURE: [&str; 25] = [
    "Hardy", "Lonely", "Brave", "Adamant", "Naughty", "Bold", "Docile", "Relaxed", "Impish", "Lax",
    "Timid", "Hasty", "Serious", "Jolly", "Naive", "Modest", "Mild", "Quiet", "Bashful", "Rash",
    "Calm", "Gentle", "Sassy", "Careful", "Quirky",
];

// Attack[0] Defense[1] Speed[2] Sp Attack[3] Sp Defense[4]
pub const NATURE_MODIFIER: [[f32; 5]; 25] = [
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

const DB: &[u8] = include_bytes!("../pk_edit.db");

pub fn extract_db() -> std::io::Result<()> {
    let mut f = File::create_new("./pk_edit.db")?;
    f.write_all(DB)?;
    Ok(())
}

pub fn held_items() -> Result<Vec<String>> {
    let conn = Connection::open("pk_edit.db")?;

    let mut stmt =
        conn.prepare("SELECT e_name FROM Items WHERE id_g3 IS NOT NULL AND type != 'Key Items'")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut res = Vec::new();
    for result in rows {
        res.push(result?);
    }
    res.push(String::from("Nothing"));

    stmt.finalize()?;

    let _ = conn.close();

    Ok(res)
}

pub fn items() -> Result<Vec<String>> {
    let conn = Connection::open("pk_edit.db")?;

    let mut stmt = conn.prepare("SELECT e_name FROM Items WHERE id_g3 IS NOT NULL AND type != 'Key Items' AND type != 'Pokeballs' AND type != 'Berries' AND type != 'Machines'")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut res = Vec::new();
    for result in rows {
        res.push(result?);
    }
    res.push(String::from("Nothing"));

    stmt.finalize()?;

    let _ = conn.close();

    Ok(res)
}

pub fn balls() -> Result<Vec<String>> {
    let conn = Connection::open("pk_edit.db")?;

    let mut stmt =
        conn.prepare("SELECT e_name FROM Items WHERE id_g3 IS NOT NULL AND type == 'Pokeballs'")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut res = Vec::new();
    for result in rows {
        res.push(result?);
    }
    res.push(String::from("Nothing"));

    stmt.finalize()?;

    let _ = conn.close();

    Ok(res)
}

pub fn berries() -> Result<Vec<String>> {
    let conn = Connection::open("pk_edit.db")?;

    let mut stmt =
        conn.prepare("SELECT e_name FROM Items WHERE id_g3 IS NOT NULL AND type == 'Berries'")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut res = Vec::new();
    for result in rows {
        res.push(result?);
    }
    res.push(String::from("Nothing"));

    stmt.finalize()?;

    let _ = conn.close();

    Ok(res)
}

pub fn tms() -> Result<Vec<String>> {
    let conn = Connection::open("pk_edit.db")?;

    let mut stmt =
        conn.prepare("SELECT e_name FROM Items WHERE id_g3 IS NOT NULL AND type == 'Machines'")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut res = Vec::new();
    for result in rows {
        res.push(result?);
    }
    res.push(String::from("Nothing"));

    stmt.finalize()?;

    let _ = conn.close();

    Ok(res)
}

pub fn key_items() -> Result<Vec<String>> {
    let conn = Connection::open("pk_edit.db")?;

    let mut stmt =
        conn.prepare("SELECT e_name FROM Items WHERE id_g3 IS NOT NULL AND type == 'Key Items'")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut res = Vec::new();
    for result in rows {
        res.push(result?);
    }
    res.push(String::from("Nothing"));

    stmt.finalize()?;

    let _ = conn.close();

    Ok(res)
}

pub fn find_item(id_g3: usize) -> Result<String> {
    let conn = Connection::open("pk_edit.db")?;

    let res = conn.query_row(
        "SELECT e_name FROM Items WHERE id_g3 = ?1",
        [id_g3],
        |row| row.get(0),
    );

    let _ = conn.close();

    res
}

pub fn item_id(name: &str) -> Result<usize> {
    let conn = Connection::open("pk_edit.db")?;
    let name = match_item_name(name);

    let res = conn.query_row("SELECT id FROM Items WHERE e_name = ?1", [name], |row| {
        row.get(0)
    });

    let _ = conn.close();

    res
}

pub fn item_id_g3(name: &str) -> Result<u16> {
    let conn = Connection::open("pk_edit.db")?;
    let name = match_item_name(name);

    let res = conn.query_row("SELECT id_g3 FROM Items WHERE e_name = ?1", [name], |row| {
        row.get(0)
    });

    let _ = conn.close();

    res
}

pub fn nat_dex_num(species: &str) -> Result<u16> {
    let conn = Connection::open("pk_edit.db")?;

    let res = conn.query_row(
        "SELECT dex_num FROM Pokedex WHERE e_name like ?1",
        [species],
        |row| row.get(0),
    );

    let _ = conn.close();

    res
}

pub fn growth_rate(dex_num: u16) -> Result<String> {
    let conn = Connection::open("pk_edit.db")?;

    let res = conn.query_row(
        "SELECT growth_rate FROM Pokedex WHERE dex_num = ?1",
        [dex_num],
        |row| row.get(0),
    );

    let _ = conn.close();

    res
}

pub fn pk_species(dex_num: u16) -> Result<String> {
    let conn = Connection::open("pk_edit.db")?;

    let res = conn.query_row(
        "SELECT e_name FROM Pokedex WHERE dex_num = ?1",
        [dex_num],
        |row| row.get(0),
    );

    let _ = conn.close();

    res
}

pub fn move_data(id: usize) -> Result<(String, String, u8)> {
    let conn = Connection::open("pk_edit.db")?;

    let res = conn.query_row(
        "SELECT type, e_name, pp FROM Moves WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    );

    let _ = conn.close();

    res
}

pub fn typing(dex_num: u16) -> Result<(String, Option<String>)> {
    let conn = Connection::open("pk_edit.db")?;

    let res = conn.query_row(
        "SELECT type1, type2 FROM Pokedex WHERE dex_num = ?1",
        [dex_num],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    let _ = conn.close();

    res
}

pub fn gender_ratio(dex_num: u16) -> Result<String> {
    let conn = Connection::open("pk_edit.db")?;

    let res = conn.query_row(
        "SELECT gender_ratio FROM Pokedex WHERE dex_num = ?1",
        [dex_num],
        |row| row.get(0),
    );

    let _ = conn.close();

    res
}

pub fn ability(dex_num: u16) -> Result<String> {
    let conn = Connection::open("pk_edit.db")?;

    let res = conn.query_row(
        "SELECT ability FROM Pokedex WHERE dex_num = ?1",
        [dex_num],
        |row| row.get(0),
    );

    let _ = conn.close();

    res
}

pub fn hidden_ability(dex_num: u16) -> Result<String> {
    let conn = Connection::open("pk_edit.db")?;

    let res = conn.query_row(
        "SELECT hidden_ability FROM Pokedex WHERE dex_num = ?1",
        [dex_num],
        |row| row.get(0),
    );

    let _ = conn.close();

    res
}

fn match_item_name(name: &str) -> &str {
    match name {
        "Parlyz Heal" => "Paralyze Heal",
        "X Defend" => "X Defense",
        "Thunderstone" => "Thunder Stone",
        "BlackGlasses" => "Black Glasses",
        "NeverMeltIce" => "Never-Melt Ice",
        "TwistedSpoon" => "Twisted Spoon",
        "DeepSeaTooth" => "Deep Sea Tooth",
        "DeepSeaScale" => "Deep Sea Scale",
        "SilverPowder" => "Silver Powder",
        "EnergyPowder" => "Energy Powder",
        _ => name,
    }
}

pub fn species() -> Result<Vec<String>> {
    let conn = Connection::open("pk_edit.db")?;

    let mut stmt = conn.prepare("SELECT e_name FROM Pokedex ORDER BY dex_num LIMIT 386")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut res = Vec::new();
    for result in rows {
        res.push(result?);
    }

    stmt.finalize()?;

    let _ = conn.close();

    Ok(res)
}

pub fn moves() -> Result<Vec<String>> {
    let conn = Connection::open("pk_edit.db")?;

    let mut stmt = conn.prepare("SELECT e_name FROM Moves WHERE is_g3 = true ORDER BY Id")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut res = Vec::new();
    for result in rows {
        res.push(result?);
    }

    stmt.finalize()?;

    let _ = conn.close();

    Ok(res)
}

pub fn find_move(name: &str) -> Result<(u16, u8)> {
    let conn = Connection::open("pk_edit.db")?;

    let res = conn.query_row(
        "SELECT Id, pp FROM Moves WHERE e_name = ?1",
        [name],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    let _ = conn.close();

    res
}

pub fn base_stats(dex_num: &u16) -> Result<(u16, u16, u16, u16, u16, u16)> {
    let conn = Connection::open("pk_edit.db")?;

    let res = conn.query_row(
        "SELECT hp, attack, defense, sp_attack, sp_defense, speed FROM Pokedex WHERE dex_num = ?1",
        [dex_num],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
    );

    let _ = conn.close();

    res
}

pub fn evolution(dex_num: &u16) -> anyhow::Result<Evolution, anyhow::Error> {
    let conn = Connection::open("pk_edit.db")?;

    let res: String = conn.query_row(
        "SELECT evolution FROM Pokedex WHERE dex_num = ?1",
        [dex_num],
        |row| row.get(0),
    )?;

    let _ = conn.close();

    Ok(serde_json::from_str::<Evolution>(&res)?)
}
