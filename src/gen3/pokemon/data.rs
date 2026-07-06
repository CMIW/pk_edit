//! Raw data sub-structures that make up the encrypted portion of a Gen III Pokémon.
//!
//! Each [`crate::pokemon::Pokemon`] contains 48 bytes of XOR-encrypted data split into four
//! 12-byte blocks whose order is determined by `PID % 24` (see [`BLOCK_ORDERS`]):
//!
//! | Block | Contents |
//! |---|---|
//! | [`GrowthBlock`] | Species, held item, experience, PP bonuses, friendship |
//! | [`AttacksBlock`] | Up to 4 move IDs and their current PP |
//! | [`EVsBlock`] | Effort values for all 6 stats + contest stats |
//! | [`MiscBlock`] | Pokérus, met location, origins, IVs, ribbons |

//! Pokémon stat calculation and IV/EV management.
//!
//! [`Stats`] stores the base stats, IVs, EVs, and nature modifiers for one Pokémon.
//! Call [`Pokemon::init_stats`] after loading a Pokémon (or changing its species/level/nature)
//! to populate these fields from the database and the encrypted sub-structure data.
//!
//! Stat formulas follow the standard Gen III rules:
//! - **HP**: `((2 × base + IV + EV÷4) × level) ÷ 100 + level + 10`
//! - **Other**: `(((2 × base + IV + EV÷4) × level) ÷ 100 + 5) × nature_modifier`

use crate::common::types::Evolution;
use crate::gen3::game_data::{Gen3GameData, EXPERIENCE_TABLE, NATURE_MODIFIER};
use crate::gen3::pokemon::pokemon::Gen3Pokemon;
use crate::traits::game_data::GameData;
use crate::traits::pokemon::Pokemon;
use byteorder::{ByteOrder, LittleEndian};
use modular_bitfield::prelude::*;
use std::fmt;

/// The Growth sub-structure (12 bytes): species, held item, experience, PP bonuses, friendship.
#[derive(Default, Copy, Clone, Debug)]
pub struct GrowthBlock {
    /// Internal Gen III species ID (not National Dex number for species ≥ 252).
    pub species: u16,
    /// Gen III item ID of the held item (`0` = nothing).
    pub item: u16,
    /// Total accumulated experience points.
    pub experience: u32,
    /// PP-Up bonus bits packed per move slot.
    pub pp_bonuses: u8,
    /// Friendship / happiness value (0–255).
    pub friendship: u8,
    /// Unused padding.
    pub unknown: u16,
}

/// The Attacks sub-structure (12 bytes): up to four move IDs and their current PP.
#[derive(Default, Copy, Clone, Debug)]
pub struct AttacksBlock {
    /// Move IDs for slots 0–3 (`0` = empty slot).
    pub moves: [u16; 4],
    /// Current PP remaining for each move slot.
    pub pp: [u8; 4],
}

/// The EVs sub-structure (12 bytes): effort values for battle stats and contest stats.
#[derive(Default, Copy, Clone, Debug)]
pub struct EVsBlock {
    /// HP effort value (0–255, total across all 6 stats ≤ 510).
    pub hp: u8,
    /// Attack EV.
    pub attack: u8,
    /// Defense EV.
    pub defense: u8,
    /// Speed EV.
    pub speed: u8,
    /// Special Attack EV.
    pub sp_attack: u8,
    /// Special Defense EV.
    pub sp_defense: u8,
    /// Contest coolness stat.
    pub coolness: u8,
    /// Contest beauty stat.
    pub beauty: u8,
    /// Contest cuteness stat.
    pub cuteness: u8,
    /// Contest smartness stat.
    pub smartness: u8,
    /// Contest toughness stat.
    pub toughness: u8,
    /// Pokéblock feel stat.
    pub feel: u8,
}

#[bitfield]
#[derive(Default, Copy, Clone, Debug)]
pub struct OriginsInfo {
    pub level_met: B7,
    pub game_of_origin: B4,
    pub pokeball: B4,
    pub ot_gender: B1,
}

#[bitfield]
#[derive(Default, Copy, Clone, Debug)]
pub struct IVsEggAbility {
    pub hp_iv: B5,
    pub attack_iv: B5,
    pub defense_iv: B5,
    pub speed_iv: B5,
    pub sp_attack_iv: B5,
    pub sp_defense_iv: B5,
    pub is_egg: B1,
    pub ability_bit: B1,
}

#[bitfield]
#[derive(Default, Copy, Clone, Debug)]
pub struct RibbonsObedience {
    pub cool_ribbons: B3,
    pub beauty_ribbons: B3,
    pub cute_ribbons: B3,
    pub smart_ribbons: B3,
    pub tough_ribbons: B3,
    pub champion_ribbon: B1,
    pub winning_ribbon: B1,
    pub victory_ribbon: B1,
    pub artist_ribbon: B1,
    pub effort_ribbon: B1,
    pub marine_ribbon: B1,
    pub land_ribbon: B1,
    pub sky_ribbon: B1,
    pub country_ribbon: B1,
    pub national_ribbon: B1,
    pub earth_ribbon: B1,
    pub world_ribbon: B1,
    pub padding: B4,
    pub obedience: B1,
}

/// The Misc sub-structure (12 bytes): Pokérus, met location, origins, IVs, and ribbons.
#[derive(Default, Copy, Clone, Debug)]
pub struct MiscBlock {
    /// Pokérus byte: upper nibble = strain (1–15), lower nibble = days remaining (0 = cured).
    pub pokerus: u8,
    /// Map location ID where the Pokémon was met.
    pub met_location: u8,
    /// Packed bitfield: level met, game of origin, Pokéball, OT gender.
    pub origins_info: OriginsInfo,
    /// Packed bitfield: six IVs (5 bits each), is-egg flag, ability bit.
    pub iv_egg_ability: IVsEggAbility,
    /// Packed bitfield: contest ribbons, battle ribbons, obedience flag.
    pub ribbons_obedience: RibbonsObedience,
}

#[derive(Default, Copy, Clone, Debug)]
pub(crate) struct PokemonData {
    pub growth: GrowthBlock,
    pub attacks: AttacksBlock,
    pub evs: EVsBlock,
    pub misc: MiscBlock,
}

#[rustfmt::skip]
#[derive(Copy, Clone, Debug)]
pub(crate) enum BlockType { Growth, Attacks, EVs, Misc }

#[rustfmt::skip]
/// The 24 permutations of the data blocks (PID % 24).
pub const BLOCK_ORDERS: [[BlockType; 4]; 24] = [
    [BlockType::Growth, BlockType::Attacks, BlockType::EVs, BlockType::Misc], // 0
    [BlockType::Growth, BlockType::Attacks, BlockType::Misc, BlockType::EVs], // 1
    [BlockType::Growth, BlockType::EVs, BlockType::Attacks, BlockType::Misc], // 2
    [BlockType::Growth, BlockType::EVs, BlockType::Misc, BlockType::Attacks], // 3
    [BlockType::Growth, BlockType::Misc, BlockType::Attacks, BlockType::EVs], // 4
    [BlockType::Growth, BlockType::Misc, BlockType::EVs, BlockType::Attacks], // 5
    [BlockType::Attacks, BlockType::Growth, BlockType::EVs, BlockType::Misc], // 6
    [BlockType::Attacks, BlockType::Growth, BlockType::Misc, BlockType::EVs], // 7
    [BlockType::Attacks, BlockType::EVs, BlockType::Growth, BlockType::Misc], // 8
    [BlockType::Attacks, BlockType::EVs, BlockType::Misc, BlockType::Growth], // 9
    [BlockType::Attacks, BlockType::Misc, BlockType::Growth, BlockType::EVs], // 10
    [BlockType::Attacks, BlockType::Misc, BlockType::EVs, BlockType::Growth], // 11
    [BlockType::EVs, BlockType::Growth, BlockType::Attacks, BlockType::Misc], // 12
    [BlockType::EVs, BlockType::Growth, BlockType::Misc, BlockType::Attacks], // 13
    [BlockType::EVs, BlockType::Attacks, BlockType::Growth, BlockType::Misc], // 14
    [BlockType::EVs, BlockType::Attacks, BlockType::Misc, BlockType::Growth], // 15
    [BlockType::EVs, BlockType::Misc, BlockType::Growth, BlockType::Attacks], // 16
    [BlockType::EVs, BlockType::Misc, BlockType::Attacks, BlockType::Growth], // 17
    [BlockType::Misc, BlockType::Growth, BlockType::Attacks, BlockType::EVs], // 18
    [BlockType::Misc, BlockType::Growth, BlockType::EVs, BlockType::Attacks], // 19
    [BlockType::Misc, BlockType::Attacks, BlockType::Growth, BlockType::EVs], // 20
    [BlockType::Misc, BlockType::Attacks, BlockType::EVs, BlockType::Growth], // 21
    [BlockType::Misc, BlockType::EVs, BlockType::Growth, BlockType::Attacks], // 22
    [BlockType::Misc, BlockType::EVs, BlockType::Attacks, BlockType::Growth], // 23
];

/// The language code stored in a Pokémon's header byte (offset 0x12).
#[derive(Debug)]
pub enum Language {
    /// Language code 1.
    Japanese,
    /// Language code 2.
    English,
    /// Language code 3.
    French,
    /// Language code 4.
    Italian,
    /// Language code 5.
    German,
    /// Any code not in 1–7 (treated as no language).
    Unused,
    /// Language code 7.
    Spanish,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Unused => write!(f, ""),
            Language::French => write!(f, "FRE"),
            Language::German => write!(f, "GER"),
            Language::English => write!(f, "ENG"),
            Language::Italian => write!(f, "ITA"),
            Language::Spanish => write!(f, "SPA"),
            Language::Japanese => write!(f, "JAP"),
        }
    }
}

impl From<u8> for Language {
    fn from(language: u8) -> Self {
        match language {
            1 => Language::Japanese,
            2 => Language::English,
            3 => Language::French,
            4 => Language::Italian,
            5 => Language::German,
            7 => Language::Spanish,
            _ => Language::Unused,
        }
    }
}

impl Evolution {
    /// Returns the level required for this evolution, if the method is `"Level"`.
    pub fn level_condition(&self) -> Option<u8> {
        println!("Level condition: {:?}", self.condition.as_ref());
        println!("Method: {:?}", self.method.as_ref());
        if self.method.as_deref() == Some("Level") {
            self.condition.as_ref().and_then(|c| c.parse::<u8>().ok())
        } else {
            None
        }
    }
}

/// Calculated stats, IVs, EVs, and nature modifiers for a [`crate::pokemon::Pokemon`].
///
/// The base stat fields (`hp`, `attack`, …) are populated from the Pokédex database.
/// The IV and EV fields mirror the encrypted sub-structure. Call each stat method with the
/// current level to obtain the in-game stat value.
#[derive(Debug, Default, Copy, Clone)]
pub struct Stats {
    // --- Base stats (from Pokédex database) ---
    pub(crate) hp: u16,
    pub(crate) attack: u16,
    pub(crate) defense: u16,
    pub(crate) sp_attack: u16,
    pub(crate) sp_defense: u16,
    pub(crate) speed: u16,
    // --- Effort Values (0–252 each, total ≤ 510) ---
    pub hp_ev: u16,
    pub attack_ev: u16,
    pub defense_ev: u16,
    pub sp_attack_ev: u16,
    pub sp_defense_ev: u16,
    pub speed_ev: u16,
    // --- Individual Values (0–31 each) ---
    pub hp_iv: u16,
    pub attack_iv: u16,
    pub defense_iv: u16,
    pub sp_attack_iv: u16,
    pub sp_defense_iv: u16,
    pub speed_iv: u16,
    // --- Nature modifiers [Atk, Def, Spd, SpAtk, SpDef] (0.9 / 1.0 / 1.1) ---
    n_mod: [f32; 5],
}

impl Stats {
    pub fn hp(&self, level: u8) -> u16 {
        let level: u16 = level as u16;

        (((2 * self.hp + self.hp_iv + (self.hp_ev / 4)) * level) / 100) + level + 10
    }

    pub fn attack(&self, level: u8) -> u16 {
        calc_stat(
            self.attack,
            self.attack_iv,
            self.attack_ev,
            self.n_mod[0],
            level,
        )
    }

    pub fn defense(&self, level: u8) -> u16 {
        calc_stat(
            self.defense,
            self.defense_iv,
            self.defense_ev,
            self.n_mod[1],
            level,
        )
    }

    pub fn speed(&self, level: u8) -> u16 {
        calc_stat(
            self.speed,
            self.speed_iv,
            self.speed_ev,
            self.n_mod[2],
            level,
        )
    }

    pub fn sp_attack(&self, level: u8) -> u16 {
        calc_stat(
            self.sp_attack,
            self.sp_attack_iv,
            self.sp_attack_ev,
            self.n_mod[3],
            level,
        )
    }

    pub fn sp_defense(&self, level: u8) -> u16 {
        calc_stat(
            self.sp_defense,
            self.sp_defense_iv,
            self.sp_defense_ev,
            self.n_mod[4],
            level,
        )
    }

    pub fn highest_stat(&self, level: u8) -> (&'static str, u16) {
        let mut stats = [
            ("HP", self.hp(level)),
            ("Attack", self.attack(level)),
            ("Defense", self.defense(level)),
            ("Sp. Attack", self.sp_attack(level)),
            ("Sp. Defense", self.sp_defense(level)),
            ("Speed", self.speed(level)),
        ];

        stats.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        stats[0]
    }

    pub fn update_ivs(&mut self, iv: &str, new_iv: u16) {
        match iv {
            "HP" => {
                self.hp_iv = recalc_iv(new_iv);
            }
            "Attack" => {
                self.attack_iv = recalc_iv(new_iv);
            }
            "Defense" => {
                self.defense_iv = recalc_iv(new_iv);
            }
            "Sp. Atk" => {
                self.sp_attack_iv = recalc_iv(new_iv);
            }
            "Sp. Def" => {
                self.sp_defense_iv = recalc_iv(new_iv);
            }
            "Speed" => {
                self.speed_iv = recalc_iv(new_iv);
            }
            _ => {}
        }
    }

    pub fn update_evs(&mut self, ev: &str, new_ev: u16) {
        match ev {
            "HP" => {
                let new_total = new_ev
                    + self.attack_ev
                    + self.defense_ev
                    + self.sp_attack_ev
                    + self.sp_defense_ev
                    + self.speed_ev;

                self.hp_ev = recalc_ev(new_ev, new_total);
            }
            "Attack" => {
                let new_total = new_ev
                    + self.hp_ev
                    + self.defense_ev
                    + self.sp_attack_ev
                    + self.sp_defense_ev
                    + self.speed_ev;

                self.attack_ev = recalc_ev(new_ev, new_total);
            }
            "Defense" => {
                let new_total = new_ev
                    + self.hp_ev
                    + self.attack_ev
                    + self.sp_attack_ev
                    + self.sp_defense_ev
                    + self.speed_ev;

                self.defense_ev = recalc_ev(new_ev, new_total);
            }
            "Sp. Atk" => {
                let new_total = new_ev
                    + self.hp_ev
                    + self.attack_ev
                    + self.defense_ev
                    + self.sp_defense_ev
                    + self.speed_ev;

                self.sp_attack_ev = recalc_ev(new_ev, new_total);
            }
            "Sp. Def" => {
                let new_total = new_ev
                    + self.hp_ev
                    + self.attack_ev
                    + self.defense_ev
                    + self.sp_attack_ev
                    + self.speed_ev;

                self.sp_defense_ev = recalc_ev(new_ev, new_total);
            }
            "Speed" => {
                let new_total = new_ev
                    + self.hp_ev
                    + self.attack_ev
                    + self.defense_ev
                    + self.sp_attack_ev
                    + self.sp_defense_ev;

                self.speed_ev = recalc_ev(new_ev, new_total);
            }
            _ => {}
        }
    }
}

fn calc_stat(base: u16, iv: u16, ev: u16, n_mod: f32, level: u8) -> u16 {
    let level: u16 = level as u16;
    (((((2 * base + iv + (ev / 4)) * level) / 100) + 5) as f32 * n_mod).floor() as u16
}

fn recalc_ev(new_ev: u16, new_total: u16) -> u16 {
    if new_total < 510 && new_ev < 252 {
        new_ev
    } else if new_total < 510 && new_ev > 252 {
        new_ev.saturating_sub(new_ev.saturating_sub(252))
    } else {
        new_ev.saturating_sub(new_total.saturating_sub(510))
    }
}

fn recalc_iv(new_iv: u16) -> u16 {
    if new_iv < 31 {
        new_iv
    } else {
        new_iv.saturating_sub(new_iv.saturating_sub(31))
    }
}

impl Gen3Pokemon {
    pub fn init_stats(&mut self) {
        if let Ok(base_stats) = Gen3GameData.base_stats(&self.nat_dex_number()) {
            let evs = &self.data.evs;
            let ivs = self.data.misc.iv_egg_ability;
            let nature_idx = (self.personality_value % 25) as usize;

            self.stats = Stats {
                // Base
                hp: base_stats.0,
                attack: base_stats.1,
                defense: base_stats.2,
                sp_attack: base_stats.3,
                sp_defense: base_stats.4,
                speed: base_stats.5,
                hp_ev: evs.hp as u16,
                attack_ev: evs.attack as u16,
                defense_ev: evs.defense as u16,
                sp_attack_ev: evs.sp_attack as u16,
                sp_defense_ev: evs.sp_defense as u16,
                speed_ev: evs.speed as u16,
                hp_iv: ivs.hp_iv() as u16,
                attack_iv: ivs.attack_iv() as u16,
                defense_iv: ivs.defense_iv() as u16,
                sp_attack_iv: ivs.sp_attack_iv() as u16,
                sp_defense_iv: ivs.sp_defense_iv() as u16,
                speed_iv: ivs.speed_iv() as u16,
                n_mod: NATURE_MODIFIER[nature_idx],
            };
        }
    }
}

pub(crate) fn growth_slice_to_u16_array(slice: &[u8], out: &mut [u16; 4]) {
    for i in 0..4 {
        out[i] = LittleEndian::read_u16(&slice[i * 2..(i * 2) + 2]);
    }
}

pub(crate) fn growth_index(growth: &str) -> usize {
    match growth {
        "Erratic" => 0,
        "Fast" => 1,
        "Medium Fast" => 2,
        "Medium Slow" => 3,
        "Slow" => 4,
        "Fluctuating" => 5,
        _ => 0,
    }
}

pub(crate) fn find_level(target: u32, index: usize) -> u32 {
    let mut lo: usize = 0;
    let mut hi: usize = 99;
    let mut res: u32 = 1;
    loop {
        if lo > hi {
            break;
        }
        let mid = lo + (hi - lo) / 2;
        if let Some(row) = EXPERIENCE_TABLE.get(mid) {
            let value = row[index];
            if value <= target {
                res = row[6];
                lo = mid + 1;
            } else if mid == 0 {
                break;
            } else {
                hi = mid - 1;
            }
        } else {
            break;
        }
    }
    res
}
