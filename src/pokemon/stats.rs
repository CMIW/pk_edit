//! Pokémon stat calculation and IV/EV management.
//!
//! [`Stats`] stores the base stats, IVs, EVs, and nature modifiers for one Pokémon.
//! Call [`Pokemon::init_stats`] after loading a Pokémon (or changing its species/level/nature)
//! to populate these fields from the database and the encrypted sub-structure data.
//!
//! Stat formulas follow the standard Gen III rules:
//! - **HP**: `((2 × base + IV + EV÷4) × level) ÷ 100 + level + 10`
//! - **Other**: `(((2 × base + IV + EV÷4) × level) ÷ 100 + 5) × nature_modifier`

use crate::misc::base_stats;
use crate::misc::NATURE_MODIFIER;
use crate::pokemon::Pokemon;

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

impl Pokemon {
    pub fn init_stats(&mut self) {
        if let Ok(base_stats) = base_stats(&self.nat_dex_number()) {
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
