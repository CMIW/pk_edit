//! Seed `pk_edit.db` from PKLumiHex and Veekun source data.
//!
//! # Usage
//!
//! ```text
//! cargo run --bin seed_db -- \
//!     --pkhex  /path/to/PKLumiHex \
//!     --veekun /path/to/veekun/pokedex \
//!     --output /path/to/pk_edit.db
//! ```
//!
//! # Data sources
//!
//! - **PKLumiHex** — personal info binaries (species stats, ability/type IDs,
//!   gender ratio), text files (species/ability/type/item names per language)
//! - **Veekun** — `moves.csv` (power, pp, accuracy, type), `move_names.csv`,
//!   `item_game_indices.csv` (per-gen item IDs), item categories, item flags
//!
//! # Limitations
//!
//! - Move stats use Veekun's latest-generation values; historical per-gen
//!   differences are not modelled.
//! - Veekun covers Gen I–VI moves only. Gen VII–IX moves are seeded with
//!   names only (stats NULL).
//! - Item pocket assignment uses Veekun's canonical pocket mapping.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const LANGS: &[&str] = &["en", "zh", "ja", "fr", "de", "es", "it", "ko"];

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS types (
    id          INTEGER NOT NULL,
    game_family TEXT    NOT NULL,
    name_en     TEXT    NOT NULL,
    name_zh     TEXT, name_ja TEXT, name_fr TEXT,
    name_de     TEXT, name_es TEXT, name_it TEXT, name_ko TEXT,
    PRIMARY KEY (id, game_family)
);
CREATE TABLE IF NOT EXISTS abilities (
    id_in_game  INTEGER NOT NULL,
    game_family TEXT    NOT NULL,
    name_en     TEXT    NOT NULL,
    name_zh     TEXT, name_ja TEXT, name_fr TEXT,
    name_de     TEXT, name_es TEXT, name_it TEXT, name_ko TEXT,
    PRIMARY KEY (id_in_game, game_family)
);
CREATE TABLE IF NOT EXISTS species (
    dex_num        INTEGER NOT NULL,
    game_family    TEXT    NOT NULL,
    name_en        TEXT    NOT NULL,
    name_zh        TEXT, name_ja TEXT, name_fr TEXT,
    name_de        TEXT, name_es TEXT, name_it TEXT, name_ko TEXT,
    type1          INTEGER NOT NULL,
    type2          INTEGER,
    hp             INTEGER NOT NULL,
    atk            INTEGER NOT NULL,
    def            INTEGER NOT NULL,
    spa            INTEGER NOT NULL,
    spd            INTEGER NOT NULL,
    spe            INTEGER NOT NULL,
    ability1       INTEGER NOT NULL,
    ability2       INTEGER,
    hidden_ability INTEGER,
    gender_ratio   INTEGER NOT NULL,
    growth_rate    TEXT    NOT NULL,
    id_in_game     INTEGER NOT NULL,
    PRIMARY KEY (dex_num, game_family)
);
CREATE TABLE IF NOT EXISTS moves (
    id_in_game  INTEGER NOT NULL,
    game_family TEXT    NOT NULL,
    name_en     TEXT    NOT NULL,
    name_zh     TEXT, name_ja TEXT, name_fr TEXT,
    name_de     TEXT, name_es TEXT, name_it TEXT, name_ko TEXT,
    type        INTEGER,
    power       INTEGER,
    pp          INTEGER,
    accuracy    INTEGER,
    PRIMARY KEY (id_in_game, game_family)
);
CREATE TABLE IF NOT EXISTS items (
    id_in_game  INTEGER NOT NULL,
    game_family TEXT    NOT NULL,
    name_en     TEXT    NOT NULL,
    name_zh     TEXT, name_ja TEXT, name_fr TEXT,
    name_de     TEXT, name_es TEXT, name_it TEXT, name_ko TEXT,
    pocket      TEXT    NOT NULL,
    holdable    INTEGER NOT NULL,
    sprite_id   INTEGER,
    PRIMARY KEY (id_in_game, game_family)
);
CREATE TABLE IF NOT EXISTS evolutions (
    dex_num      INTEGER NOT NULL,
    game_family  TEXT    NOT NULL,
    evolves_into INTEGER,
    method       TEXT,
    condition    TEXT,
    PRIMARY KEY (dex_num, game_family)
);
";

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Args {
    pkhex: PathBuf,
    veekun: PathBuf,
    output: PathBuf,
}

fn parse_args() -> Result<Args> {
    let mut args = std::env::args().skip(1);
    let mut pkhex = None::<PathBuf>;
    let mut veekun = None::<PathBuf>;
    let mut output = None::<PathBuf>;

    while let Some(flag) = args.next() {
        let value = args
            .next()
            .with_context(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--pkhex" => pkhex = Some(PathBuf::from(value)),
            "--veekun" => veekun = Some(PathBuf::from(value)),
            "--output" => output = Some(PathBuf::from(value)),
            other => bail!("unknown argument: {other}"),
        }
    }

    Ok(Args {
        pkhex: pkhex.context("--pkhex is required")?,
        veekun: veekun.context("--veekun is required")?,
        output: output.context("--output is required")?,
    })
}

// ---------------------------------------------------------------------------
// Names helpers
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Names {
    en: String,
    zh: Option<String>,
    ja: Option<String>,
    fr: Option<String>,
    de: Option<String>,
    es: Option<String>,
    it: Option<String>,
    ko: Option<String>,
}

fn read_text_file(path: &Path) -> Result<Vec<String>> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;

    // Detect UTF-16 LE BOM (FF FE)
    let text = if bytes.starts_with(&[0xFF, 0xFE]) {
        let words: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&words)
            .with_context(|| format!("UTF-16 decode of {}", path.display()))?
    } else {
        let s = std::str::from_utf8(&bytes)
            .with_context(|| format!("UTF-8 decode of {}", path.display()))?;
        s.strip_prefix('\u{FEFF}').unwrap_or(s).to_string()
    };

    Ok(text.lines().map(str::to_string).collect())
}

fn load_pkhex_names(pkhex_root: &Path, template: &str) -> HashMap<&'static str, Vec<String>> {
    let mut result = HashMap::new();
    for &lang in LANGS {
        let subpath = template.replace("{lang}", lang);
        let path = pkhex_root.join(subpath);
        match read_text_file(&path) {
            Ok(lines) => {
                result.insert(lang, lines);
            }
            Err(e) => eprintln!("[warn] {e}"),
        }
    }
    result
}

fn get_names(by_lang: &HashMap<&str, Vec<String>>, idx: usize) -> Names {
    let get = |lang: &str| -> Option<String> {
        let s = by_lang.get(lang)?.get(idx)?;
        if s.is_empty() || s == "---" || s == "\u{2014}" || s == "(None)" {
            None
        } else {
            Some(s.clone())
        }
    };
    Names {
        en: get("en").unwrap_or_else(|| format!("[{idx}]")),
        zh: get("zh"),
        ja: get("ja"),
        fr: get("fr"),
        de: get("de"),
        es: get("es"),
        it: get("it"),
        ko: get("ko"),
    }
}

// ---------------------------------------------------------------------------
// Byte reading helpers
// ---------------------------------------------------------------------------

fn read_u8_at(data: &[u8], offset: usize) -> Result<u8> {
    data.get(offset)
        .copied()
        .ok_or_else(|| anyhow::anyhow!("byte at 0x{offset:02X} out of bounds (len={})", data.len()))
}

fn read_u16_le_at(data: &[u8], offset: usize) -> Result<u16> {
    let slice = data
        .get(offset..offset + 2)
        .ok_or_else(|| anyhow::anyhow!("u16 at 0x{offset:02X} out of bounds"))?;
    let arr: [u8; 2] = slice
        .try_into()
        .map_err(|_| anyhow::anyhow!("slice/array mismatch at 0x{offset:02X}"))?;
    Ok(u16::from_le_bytes(arr))
}

// ---------------------------------------------------------------------------
// Personal info parsing
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct PersonalEntry {
    hp: u8,
    atk: u8,
    def: u8,
    spe: u8,
    spa: u8,
    spd: u8,
    type1: u8,
    type2: u8,
    gender_ratio: u8,
    growth_rate: u8,
    ability1: u16,
    ability2: Option<u16>,
    hidden_ability: Option<u16>,
}

fn parse_gen3(data: &[u8]) -> Result<PersonalEntry> {
    let ability1 = u16::from(read_u8_at(data, 0x16)?);
    let a2 = u16::from(read_u8_at(data, 0x17)?);
    Ok(PersonalEntry {
        hp: read_u8_at(data, 0x00)?,
        atk: read_u8_at(data, 0x01)?,
        def: read_u8_at(data, 0x02)?,
        spe: read_u8_at(data, 0x03)?,
        spa: read_u8_at(data, 0x04)?,
        spd: read_u8_at(data, 0x05)?,
        type1: read_u8_at(data, 0x06)?,
        type2: read_u8_at(data, 0x07)?,
        gender_ratio: read_u8_at(data, 0x10)?,
        growth_rate: read_u8_at(data, 0x13)?,
        ability1,
        ability2: (a2 != 0 && a2 != ability1).then_some(a2),
        hidden_ability: None,
    })
}

fn parse_gen4(data: &[u8]) -> Result<PersonalEntry> {
    let ability1 = u16::from(read_u8_at(data, 0x16)?);
    let a2 = u16::from(read_u8_at(data, 0x17)?);
    Ok(PersonalEntry {
        hp: read_u8_at(data, 0x00)?,
        atk: read_u8_at(data, 0x01)?,
        def: read_u8_at(data, 0x02)?,
        spe: read_u8_at(data, 0x03)?,
        spa: read_u8_at(data, 0x04)?,
        spd: read_u8_at(data, 0x05)?,
        type1: read_u8_at(data, 0x06)?,
        type2: read_u8_at(data, 0x07)?,
        gender_ratio: read_u8_at(data, 0x10)?,
        growth_rate: read_u8_at(data, 0x13)?,
        ability1,
        ability2: (a2 != 0 && a2 != ability1).then_some(a2),
        hidden_ability: None,
    })
}

fn parse_bdsp(data: &[u8]) -> Result<PersonalEntry> {
    let ability1 = read_u16_le_at(data, 0x18)?;
    let ability2 = read_u16_le_at(data, 0x1A)?;
    let ability_h = read_u16_le_at(data, 0x1C)?;
    // PokeDexIndex at 0x42 is the Sinnoh regional dex number — metadata only,
    // not used as dex_num. The array index is the national dex number.
    Ok(PersonalEntry {
        hp: read_u8_at(data, 0x00)?,
        atk: read_u8_at(data, 0x01)?,
        def: read_u8_at(data, 0x02)?,
        spe: read_u8_at(data, 0x03)?,
        spa: read_u8_at(data, 0x04)?,
        spd: read_u8_at(data, 0x05)?,
        type1: read_u8_at(data, 0x06)?,
        type2: read_u8_at(data, 0x07)?,
        gender_ratio: read_u8_at(data, 0x12)?,
        growth_rate: read_u8_at(data, 0x15)?,
        ability1,
        ability2: (ability2 != 0 && ability2 != ability1).then_some(ability2),
        hidden_ability: (ability_h != 0 && ability_h != ability1 && ability_h != ability2)
            .then_some(ability_h),
    })
}

fn growth_rate_name(byte: u8) -> &'static str {
    match byte {
        0 => "Medium Fast",
        1 => "Erratic",
        2 => "Fluctuating",
        3 => "Medium Slow",
        4 => "Fast",
        5 => "Slow",
        _ => "Medium Fast",
    }
}

// ---------------------------------------------------------------------------
// Veekun mapping helpers
// ---------------------------------------------------------------------------

fn veekun_lang(local_language_id: i64) -> Option<&'static str> {
    match local_language_id {
        9 => Some("en"),
        12 => Some("zh"),
        11 => Some("ja"),
        5 => Some("fr"),
        6 => Some("de"),
        7 => Some("es"),
        8 => Some("it"),
        3 => Some("ko"),
        _ => None,
    }
}

fn veekun_pocket(pocket_id: i64) -> &'static str {
    match pocket_id {
        1 => "items",
        2 => "medicine",
        3 => "balls",
        4 => "tms",
        5 => "berries",
        6 => "mail",
        7 => "battle",
        8 => "key",
        _ => "items",
    }
}

// ---------------------------------------------------------------------------
// CSV helpers
// ---------------------------------------------------------------------------

fn csv_col(headers: &csv::StringRecord, name: &str) -> Result<usize> {
    headers
        .iter()
        .position(|h| h == name)
        .ok_or_else(|| anyhow::anyhow!("CSV column '{name}' not found"))
}

fn csv_field<'a>(record: &'a csv::StringRecord, col: usize) -> Result<&'a str> {
    record
        .get(col)
        .ok_or_else(|| anyhow::anyhow!("CSV field at column {col} missing"))
}

fn parse_i64(s: &str) -> Result<i64> {
    s.parse::<i64>()
        .with_context(|| format!("parsing '{s}' as i64"))
}

fn parse_opt_i64(s: &str) -> Option<i64> {
    if s.is_empty() {
        None
    } else {
        s.parse::<i64>().ok()
    }
}

// ---------------------------------------------------------------------------
// Seed: types
// ---------------------------------------------------------------------------

fn seed_types(conn: &Connection, pkhex_root: &Path) -> Result<()> {
    println!("Seeding types...");
    let names = load_pkhex_names(
        pkhex_root,
        "PKHeX.Core/Resources/text/other/{lang}/text_Types_{lang}.txt",
    );
    let en = names.get("en").cloned().unwrap_or_default();
    let families = ["gen3", "gen4", "bdsp", "lumi"];
    let mut count: u32 = 0;

    for gf in &families {
        for (idx, en_name) in en.iter().enumerate() {
            if en_name.is_empty() || en_name.starts_with('[') {
                continue;
            }
            let n = get_names(&names, idx);
            let id = i64::try_from(idx).context("type id overflow")?;
            conn.execute(
                "INSERT OR REPLACE INTO types \
                 (id, game_family, name_en, name_zh, name_ja, name_fr, \
                  name_de, name_es, name_it, name_ko) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![id, gf, n.en, n.zh, n.ja, n.fr, n.de, n.es, n.it, n.ko],
            )?;
            count += 1;
        }
    }
    println!("  {count} type rows inserted.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Seed: abilities
// ---------------------------------------------------------------------------

fn seed_abilities(conn: &Connection, pkhex_root: &Path) -> Result<()> {
    println!("Seeding abilities...");
    let names = load_pkhex_names(
        pkhex_root,
        "PKHeX.Core/Resources/text/other/{lang}/text_Abilities_{lang}.txt",
    );
    let en = names.get("en").cloned().unwrap_or_default();
    let families = ["gen3", "gen4", "bdsp", "lumi"];
    let mut count: u32 = 0;

    for gf in &families {
        for (idx, en_name) in en.iter().enumerate() {
            if en_name.is_empty() || en_name.starts_with('[') {
                continue;
            }
            let n = get_names(&names, idx);
            let id = i64::try_from(idx).context("ability id overflow")?;
            conn.execute(
                "INSERT OR REPLACE INTO abilities \
                 (id_in_game, game_family, name_en, name_zh, name_ja, name_fr, \
                  name_de, name_es, name_it, name_ko) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![id, gf, n.en, n.zh, n.ja, n.fr, n.de, n.es, n.it, n.ko],
            )?;
            count += 1;
        }
    }
    println!("  {count} ability rows inserted.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Seed: species
// ---------------------------------------------------------------------------

type ParseFn = fn(&[u8]) -> Result<PersonalEntry>;

#[derive(Debug)]
struct SpeciesConfig {
    game_family: &'static str,
    filename: &'static str,
    parse: ParseFn,
    entry_size: usize,
}

const SPECIES_CONFIGS: &[SpeciesConfig] = &[
    SpeciesConfig {
        game_family: "gen3",
        filename: "personal_rs",
        parse: parse_gen3,
        entry_size: 0x1C,
    },
    SpeciesConfig {
        game_family: "gen4",
        filename: "personal_hgss",
        parse: parse_gen4,
        entry_size: 0x2C,
    },
    SpeciesConfig {
        game_family: "bdsp",
        filename: "personal_bdsp",
        parse: parse_bdsp,
        entry_size: 0x44,
    },
    SpeciesConfig {
        game_family: "lumi",
        filename: "personal_bdsplumi",
        parse: parse_bdsp,
        entry_size: 0x44,
    },
];

fn seed_species(conn: &Connection, pkhex_root: &Path) -> Result<()> {
    println!("Seeding species...");
    let species_names = load_pkhex_names(
        pkhex_root,
        "PKHeX.Core/Resources/text/other/{lang}/text_Species_{lang}.txt",
    );

    for cfg in SPECIES_CONFIGS {
        let path = pkhex_root
            .join("PKHeX.Core/Resources/byte/personal")
            .join(cfg.filename);
        let data = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;

        let entry_count = data.len() / cfg.entry_size;
        let mut count: u32 = 0;

        for i in 1..entry_count {
            let start = i * cfg.entry_size;
            let end = start + cfg.entry_size;
            let slice = data
                .get(start..end)
                .ok_or_else(|| anyhow::anyhow!("entry {i} out of bounds in {}", cfg.filename))?;
            let entry = (cfg.parse)(slice)?;

            // Array index = national dex number for all formats.
            // PokeDexIndex in BDSP/Lumi is the Sinnoh regional number — not used here.
            let dex_num = i64::try_from(i).context("dex_num overflow")?;

            let n = get_names(&species_names, i);
            if n.en.starts_with('[') {
                continue;
            }

            let type2 = (entry.type2 != entry.type1).then(|| i64::from(entry.type2));
            let growth = growth_rate_name(entry.growth_rate);
            let id_in_game = i64::try_from(i).context("id_in_game overflow")?;

            conn.execute(
                "INSERT OR REPLACE INTO species \
                 (dex_num, game_family, name_en, name_zh, name_ja, name_fr, \
                  name_de, name_es, name_it, name_ko, \
                  type1, type2, hp, atk, def, spa, spd, spe, \
                  ability1, ability2, hidden_ability, \
                  gender_ratio, growth_rate, id_in_game) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,\
                         ?11,?12,?13,?14,?15,?16,?17,?18,\
                         ?19,?20,?21,?22,?23,?24)",
                params![
                    dex_num,
                    cfg.game_family,
                    n.en,
                    n.zh,
                    n.ja,
                    n.fr,
                    n.de,
                    n.es,
                    n.it,
                    n.ko,
                    i64::from(entry.type1),
                    type2,
                    i64::from(entry.hp),
                    i64::from(entry.atk),
                    i64::from(entry.def),
                    i64::from(entry.spa),
                    i64::from(entry.spd),
                    i64::from(entry.spe),
                    i64::from(entry.ability1),
                    entry.ability2.map(i64::from),
                    entry.hidden_ability.map(i64::from),
                    i64::from(entry.gender_ratio),
                    growth,
                    id_in_game,
                ],
            )?;
            count += 1;
        }
        println!("  {}: {count} species rows inserted.", cfg.game_family);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Seed: moves
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct MoveStat {
    type_id: Option<i64>,
    power: Option<i64>,
    pp: Option<i64>,
    accuracy: Option<i64>,
}

fn load_veekun_moves(veekun_root: &Path) -> Result<HashMap<i64, MoveStat>> {
    let path = veekun_root.join("pokedex/data/csv/moves.csv");
    let mut rdr =
        csv::Reader::from_path(&path).with_context(|| format!("opening {}", path.display()))?;
    let headers = rdr.headers()?.clone();
    let col_id = csv_col(&headers, "id")?;
    let col_type = csv_col(&headers, "type_id")?;
    let col_power = csv_col(&headers, "power")?;
    let col_pp = csv_col(&headers, "pp")?;
    let col_acc = csv_col(&headers, "accuracy")?;

    let mut map = HashMap::new();
    for result in rdr.records() {
        let rec = result?;
        let id = parse_i64(csv_field(&rec, col_id)?)?;
        map.insert(
            id,
            MoveStat {
                type_id: parse_opt_i64(csv_field(&rec, col_type)?),
                power: parse_opt_i64(csv_field(&rec, col_power)?),
                pp: parse_opt_i64(csv_field(&rec, col_pp)?),
                accuracy: parse_opt_i64(csv_field(&rec, col_acc)?),
            },
        );
    }
    Ok(map)
}

fn load_veekun_move_names(
    veekun_root: &Path,
) -> Result<HashMap<i64, HashMap<&'static str, String>>> {
    let path = veekun_root.join("pokedex/data/csv/move_names.csv");
    let mut rdr =
        csv::Reader::from_path(&path).with_context(|| format!("opening {}", path.display()))?;
    let headers = rdr.headers()?.clone();
    let col_mid = csv_col(&headers, "move_id")?;
    let col_lang = csv_col(&headers, "local_language_id")?;
    let col_name = csv_col(&headers, "name")?;

    let mut map: HashMap<i64, HashMap<&'static str, String>> = HashMap::new();
    for result in rdr.records() {
        let rec = result?;
        let move_id = parse_i64(csv_field(&rec, col_mid)?)?;
        let lang_id = parse_i64(csv_field(&rec, col_lang)?)?;
        let name = csv_field(&rec, col_name)?.to_string();
        if let Some(lang) = veekun_lang(lang_id) {
            map.entry(move_id).or_default().insert(lang, name);
        }
    }
    Ok(map)
}

fn seed_moves(conn: &Connection, veekun_root: &Path, pkhex_root: &Path) -> Result<()> {
    println!("Seeding moves...");
    let veekun_stats = load_veekun_moves(veekun_root)?;
    let veekun_names = load_veekun_move_names(veekun_root)?;
    let pkhex_names = load_pkhex_names(
        pkhex_root,
        "PKHeX.Core/Resources/text/other/{lang}/text_Moves_{lang}.txt",
    );

    let en_moves = pkhex_names.get("en").cloned().unwrap_or_default();
    let families = ["gen3", "gen4", "bdsp", "lumi"];
    let mut count: u32 = 0;

    for (idx, en_name) in en_moves.iter().enumerate().skip(1) {
        if en_name.is_empty() || en_name.starts_with('[') {
            continue;
        }
        let id_in_game = i64::try_from(idx).context("move id overflow")?;

        // Names: start from PKHeX, override with Veekun where available
        let mut n = get_names(&pkhex_names, idx);
        if let Some(vn) = veekun_names.get(&id_in_game) {
            if let Some(s) = vn.get("en") {
                n.en = s.clone();
            }
            if n.zh.is_none() {
                n.zh = vn.get("zh").cloned();
            }
            if n.ja.is_none() {
                n.ja = vn.get("ja").cloned();
            }
            if n.fr.is_none() {
                n.fr = vn.get("fr").cloned();
            }
            if n.de.is_none() {
                n.de = vn.get("de").cloned();
            }
            if n.es.is_none() {
                n.es = vn.get("es").cloned();
            }
            if n.it.is_none() {
                n.it = vn.get("it").cloned();
            }
            if n.ko.is_none() {
                n.ko = vn.get("ko").cloned();
            }
        }

        let stats = veekun_stats.get(&id_in_game);

        for gf in &families {
            conn.execute(
                "INSERT OR REPLACE INTO moves \
                 (id_in_game, game_family, name_en, name_zh, name_ja, name_fr, \
                  name_de, name_es, name_it, name_ko, \
                  type, power, pp, accuracy) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
                params![
                    id_in_game,
                    gf,
                    n.en,
                    n.zh,
                    n.ja,
                    n.fr,
                    n.de,
                    n.es,
                    n.it,
                    n.ko,
                    stats.and_then(|s| s.type_id.map(|t| t - 1)),
                    stats.and_then(|s| s.power),
                    stats.and_then(|s| s.pp),
                    stats.and_then(|s| s.accuracy),
                ],
            )?;
            count += 1;
        }
    }
    println!("  {count} move rows inserted ({} unique moves).", count / 4);
    Ok(())
}

// ---------------------------------------------------------------------------
// Seed: items
// ---------------------------------------------------------------------------

fn load_veekun_item_meta(
    veekun_root: &Path,
) -> Result<(HashMap<i64, &'static str>, HashMap<i64, i64>, HashSet<i64>)> {
    // item_categories.csv → cat_id → pocket name
    let mut cat_pocket: HashMap<i64, &'static str> = HashMap::new();
    {
        let path = veekun_root.join("pokedex/data/csv/item_categories.csv");
        let mut rdr =
            csv::Reader::from_path(&path).with_context(|| format!("opening {}", path.display()))?;
        let headers = rdr.headers()?.clone();
        let col_id = csv_col(&headers, "id")?;
        let col_pocket = csv_col(&headers, "pocket_id")?;
        for result in rdr.records() {
            let rec = result?;
            let id = parse_i64(csv_field(&rec, col_id)?)?;
            let pid = parse_i64(csv_field(&rec, col_pocket)?)?;
            cat_pocket.insert(id, veekun_pocket(pid));
        }
    }

    // items.csv → item_id → category_id
    let mut item_cat: HashMap<i64, i64> = HashMap::new();
    {
        let path = veekun_root.join("pokedex/data/csv/items.csv");
        let mut rdr =
            csv::Reader::from_path(&path).with_context(|| format!("opening {}", path.display()))?;
        let headers = rdr.headers()?.clone();
        let col_id = csv_col(&headers, "id")?;
        let col_cat = csv_col(&headers, "category_id")?;
        for result in rdr.records() {
            let rec = result?;
            let id = parse_i64(csv_field(&rec, col_id)?)?;
            let cat = parse_i64(csv_field(&rec, col_cat)?)?;
            item_cat.insert(id, cat);
        }
    }

    // item_flag_map.csv → holdable item_ids (flag 5/6/7)
    let mut holdable: HashSet<i64> = HashSet::new();
    {
        let path = veekun_root.join("pokedex/data/csv/item_flag_map.csv");
        let mut rdr =
            csv::Reader::from_path(&path).with_context(|| format!("opening {}", path.display()))?;
        let headers = rdr.headers()?.clone();
        let col_item = csv_col(&headers, "item_id")?;
        let col_flag = csv_col(&headers, "item_flag_id")?;
        for result in rdr.records() {
            let rec = result?;
            let item = parse_i64(csv_field(&rec, col_item)?)?;
            let flag = parse_i64(csv_field(&rec, col_flag)?)?;
            if matches!(flag, 5 | 6 | 7) {
                holdable.insert(item);
            }
        }
    }

    Ok((cat_pocket, item_cat, holdable))
}

fn load_veekun_item_names(
    veekun_root: &Path,
) -> Result<HashMap<i64, HashMap<&'static str, String>>> {
    let path = veekun_root.join("pokedex/data/csv/item_names.csv");
    let mut rdr =
        csv::Reader::from_path(&path).with_context(|| format!("opening {}", path.display()))?;
    let headers = rdr.headers()?.clone();
    let col_item = csv_col(&headers, "item_id")?;
    let col_lang = csv_col(&headers, "local_language_id")?;
    let col_name = csv_col(&headers, "name")?;

    let mut map: HashMap<i64, HashMap<&'static str, String>> = HashMap::new();
    for result in rdr.records() {
        let rec = result?;
        let item_id = parse_i64(csv_field(&rec, col_item)?)?;
        let lang_id = parse_i64(csv_field(&rec, col_lang)?)?;
        let name = csv_field(&rec, col_name)?.to_string();
        if let Some(lang) = veekun_lang(lang_id) {
            map.entry(item_id).or_default().insert(lang, name);
        }
    }
    Ok(map)
}

fn load_veekun_game_indices(veekun_root: &Path) -> Result<HashMap<(i64, i64), i64>> {
    let path = veekun_root.join("pokedex/data/csv/item_game_indices.csv");
    let mut rdr =
        csv::Reader::from_path(&path).with_context(|| format!("opening {}", path.display()))?;
    let headers = rdr.headers()?.clone();
    let col_item = csv_col(&headers, "item_id")?;
    let col_gen = csv_col(&headers, "generation_id")?;
    let col_idx = csv_col(&headers, "game_index")?;

    let mut map = HashMap::new();
    for result in rdr.records() {
        let rec = result?;
        let item = parse_i64(csv_field(&rec, col_item)?)?;
        let gen = parse_i64(csv_field(&rec, col_gen)?)?;
        let index = parse_i64(csv_field(&rec, col_idx)?)?;
        map.insert((item, gen), index);
    }
    Ok(map)
}

fn item_pocket_holdable(
    veekun_id: Option<i64>,
    cat_pocket: &HashMap<i64, &'static str>,
    item_cat: &HashMap<i64, i64>,
    holdable: &HashSet<i64>,
) -> (&'static str, i32) {
    let pocket = veekun_id
        .and_then(|id| item_cat.get(&id))
        .and_then(|cat| cat_pocket.get(cat))
        .copied()
        .unwrap_or("items");
    let is_holdable = veekun_id.is_some_and(|id| holdable.contains(&id));
    (pocket, if is_holdable { 1 } else { 0 })
}

fn seed_items(conn: &Connection, veekun_root: &Path, pkhex_root: &Path) -> Result<()> {
    println!("Seeding items...");

    let (cat_pocket, item_cat, holdable) = load_veekun_item_meta(veekun_root)?;
    let veekun_names = load_veekun_item_names(veekun_root)?;
    let game_indices = load_veekun_game_indices(veekun_root)?;

    // Build reverse map: en_name → veekun_item_id (for name-based matching)
    let name_to_veekun: HashMap<&str, i64> = veekun_names
        .iter()
        .filter_map(|(id, langs)| langs.get("en").map(|name| (name.as_str(), *id)))
        .collect();

    // PKHeX item arrays indexed by id_in_game
    let gen3_names = load_pkhex_names(
        pkhex_root,
        "PKHeX.Core/Resources/text/items/gen3/text_ItemsG3_{lang}.txt",
    );
    let global_names = load_pkhex_names(
        pkhex_root,
        "PKHeX.Core/Resources/text/items/text_Items_{lang}.txt",
    );
    let lumi_names = load_pkhex_names(
        pkhex_root,
        "PKHeX.Core/Resources/text/items/text_ItemsLUMI_{lang}.txt",
    );

    let name_to_sv_id: HashMap<&str, usize> = global_names
        .get("en")
        .iter()
        .flat_map(|v| v.iter().enumerate().skip(1))
        .filter(|(_, name)| !name.is_empty() && !matches!(name.as_str(), "(None)" | "---"))
        .map(|(idx, name)| (name.as_str(), idx))
        .rev()
        .collect();

    let mut count: u32 = 0;

    // --- Gen III: PKHeX gen3 array, name-match Veekun for pocket/holdable ---
    let en_g3 = gen3_names.get("en").cloned().unwrap_or_default();
    for (idx, en_name) in en_g3.iter().enumerate().skip(1) {
        if en_name.is_empty() || matches!(en_name.as_str(), "(None)" | "---") {
            continue;
        }
        let id_in_game = i64::try_from(idx).context("gen3 item id overflow")?;
        let veekun_id = name_to_veekun.get(en_name.as_str()).copied();
        let (pocket, holdable_flag) =
            item_pocket_holdable(veekun_id, &cat_pocket, &item_cat, &holdable);
        let sprite_id = name_to_sv_id.get(en_name.as_str()).map(|&i| i as i64);
        let n = get_names(&gen3_names, idx);
        conn.execute(
            "INSERT OR REPLACE INTO items \
             (id_in_game, game_family, name_en, name_zh, name_ja, name_fr, \
              name_de, name_es, name_it, name_ko, pocket, holdable, sprite_id) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            params![
                id_in_game,
                "gen3",
                n.en,
                n.zh,
                n.ja,
                n.fr,
                n.de,
                n.es,
                n.it,
                n.ko,
                pocket,
                holdable_flag,
                sprite_id,
            ],
        )?;
        count += 1;
    }

    // --- Gen IV (gen_id=4) and BDSP (gen_id=8): Veekun game indices ---
    for (gf, gen_id) in [("gen4", 4_i64), ("bdsp", 8_i64)] {
        let mut seen: HashSet<i64> = HashSet::new();
        for ((item_id, gid), &game_index) in &game_indices {
            if *gid != gen_id || !seen.insert(game_index) {
                continue;
            }
            let veekun_id = Some(*item_id);
            let (pocket, holdable_flag) =
                item_pocket_holdable(veekun_id, &cat_pocket, &item_cat, &holdable);

            // Build names: Veekun first, fill gaps from PKHeX global array
            let vnames = veekun_names.get(item_id);
            let pnames = get_names(
                &global_names,
                usize::try_from(game_index).context("game_index to usize")?,
            );
            let n = Names {
                en: vnames
                    .and_then(|m| m.get("en"))
                    .cloned()
                    .unwrap_or(pnames.en),
                zh: vnames.and_then(|m| m.get("zh")).cloned().or(pnames.zh),
                ja: vnames.and_then(|m| m.get("ja")).cloned().or(pnames.ja),
                fr: vnames.and_then(|m| m.get("fr")).cloned().or(pnames.fr),
                de: vnames.and_then(|m| m.get("de")).cloned().or(pnames.de),
                es: vnames.and_then(|m| m.get("es")).cloned().or(pnames.es),
                it: vnames.and_then(|m| m.get("it")).cloned().or(pnames.it),
                ko: vnames.and_then(|m| m.get("ko")).cloned().or(pnames.ko),
            };

            let sprite_id = name_to_sv_id.get(n.en.as_str()).map(|&i| i as i64);

            conn.execute(
                "INSERT OR REPLACE INTO items \
                 (id_in_game, game_family, name_en, name_zh, name_ja, name_fr, \
                  name_de, name_es, name_it, name_ko, pocket, holdable, sprite_id) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
                params![
                    game_index,
                    gf,
                    n.en,
                    n.zh,
                    n.ja,
                    n.fr,
                    n.de,
                    n.es,
                    n.it,
                    n.ko,
                    pocket,
                    holdable_flag,
                    sprite_id,
                ],
            )?;
            count += 1;
        }
    }

    // --- Lumi: PKHeX LUMI array, name-match Veekun for pocket/holdable ---
    let en_lumi = lumi_names.get("en").cloned().unwrap_or_default();
    for (idx, en_name) in en_lumi.iter().enumerate().skip(1) {
        if en_name.is_empty() || matches!(en_name.as_str(), "(None)" | "---") {
            continue;
        }
        let id_in_game = i64::try_from(idx).context("lumi item id overflow")?;
        let veekun_id = name_to_veekun.get(en_name.as_str()).copied();
        let (pocket, holdable_flag) =
            item_pocket_holdable(veekun_id, &cat_pocket, &item_cat, &holdable);
        let sprite_id = name_to_sv_id.get(en_name.as_str()).map(|&i| i as i64);
        let n = get_names(&lumi_names, idx);
        conn.execute(
            "INSERT OR REPLACE INTO items \
             (id_in_game, game_family, name_en, name_zh, name_ja, name_fr, \
              name_de, name_es, name_it, name_ko, pocket, holdable, sprite_id) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            params![
                id_in_game,
                "lumi",
                n.en,
                n.zh,
                n.ja,
                n.fr,
                n.de,
                n.es,
                n.it,
                n.ko,
                pocket,
                holdable_flag,
                sprite_id,
            ],
        )?;
        count += 1;
    }

    println!("  {count} item rows inserted.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    let args = parse_args()?;

    if !args.pkhex.exists() {
        bail!("PKLumiHex path not found: {}", args.pkhex.display());
    }
    if !args.veekun.exists() {
        bail!("Veekun path not found: {}", args.veekun.display());
    }

    println!("Output: {}", args.output.display());

    if args.output.exists() {
        std::fs::remove_file(&args.output)
            .with_context(|| format!("removing {}", args.output.display()))?;
    }

    let conn = Connection::open(&args.output)
        .with_context(|| format!("opening {}", args.output.display()))?;

    conn.execute_batch(SCHEMA)?;

    seed_types(&conn, &args.pkhex)?;
    seed_abilities(&conn, &args.pkhex)?;
    seed_species(&conn, &args.pkhex)?;
    seed_moves(&conn, &args.veekun, &args.pkhex)?;
    seed_items(&conn, &args.veekun, &args.pkhex)?;

    println!("Done.");
    Ok(())
}
