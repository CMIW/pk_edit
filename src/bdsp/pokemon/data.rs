#[derive(Debug, Default, Clone, Copy)]
pub struct Stats {
    pub base_hp: u16,
    pub base_attack: u16,
    pub base_defense: u16,
    pub base_sp_attack: u16,
    pub base_sp_defense: u16,
    pub base_speed: u16,
    pub hp_ev: u16,
    pub attack_ev: u16,
    pub defense_ev: u16,
    pub sp_attack_ev: u16,
    pub sp_defense_ev: u16,
    pub speed_ev: u16,
    pub hp_iv: u16,
    pub attack_iv: u16,
    pub defense_iv: u16,
    pub sp_attack_iv: u16,
    pub sp_defense_iv: u16,
    pub speed_iv: u16,
    pub(crate) n_mod: [f32; 5],
}

impl Stats {
    pub fn hp(&self, level: u8) -> u16 {
        let level: u16 = level as u16;
        (((2 * self.base_hp + self.hp_iv + (self.hp_ev / 4)) * level) / 100) + level + 10
    }

    pub fn attack(&self, level: u8) -> u16 {
        calc_stat(self.base_attack, self.attack_iv, self.attack_ev, self.n_mod[0], level)
    }

    pub fn defense(&self, level: u8) -> u16 {
        calc_stat(self.base_defense, self.defense_iv, self.defense_ev, self.n_mod[1], level)
    }

    pub fn sp_attack(&self, level: u8) -> u16 {
        calc_stat(self.base_sp_attack, self.sp_attack_iv, self.sp_attack_ev, self.n_mod[3], level)
    }

    pub fn sp_defense(&self, level: u8) -> u16 {
        calc_stat(self.base_sp_defense, self.sp_defense_iv, self.sp_defense_ev, self.n_mod[4], level)
    }

    pub fn speed(&self, level: u8) -> u16 {
        calc_stat(self.base_speed, self.speed_iv, self.speed_ev, self.n_mod[2], level)
    }

    pub fn update_ivs(&mut self, iv: &str, new_iv: u16) {
        let val = recalc_iv(new_iv);
        match iv {
            "HP" => { self.hp_iv = val; }
            "Attack" => { self.attack_iv = val; }
            "Defense" => { self.defense_iv = val; }
            "Sp. Atk" => { self.sp_attack_iv = val; }
            "Sp. Def" => { self.sp_defense_iv = val; }
            "Speed" => { self.speed_iv = val; }
            _ => {}
        }
    }

    pub fn update_evs(&mut self, ev: &str, new_ev: u16) {
        let others: u16 = self.hp_ev + self.attack_ev + self.defense_ev
            + self.sp_attack_ev + self.sp_defense_ev + self.speed_ev;
        match ev {
            "HP" => { self.hp_ev = recalc_ev(new_ev, others + new_ev); }
            "Attack" => { self.attack_ev = recalc_ev(new_ev, others + new_ev); }
            "Defense" => { self.defense_ev = recalc_ev(new_ev, others + new_ev); }
            "Sp. Atk" => { self.sp_attack_ev = recalc_ev(new_ev, others + new_ev); }
            "Sp. Def" => { self.sp_defense_ev = recalc_ev(new_ev, others + new_ev); }
            "Speed" => { self.speed_ev = recalc_ev(new_ev, others + new_ev); }
            _ => {}
        }
    }
}

fn calc_stat(base: u16, iv: u16, ev: u16, n_mod: f32, level: u8) -> u16 {
    let level: u16 = level as u16;
    (((((2 * base + iv + (ev / 4)) * level) / 100) + 5) as f32 * n_mod).floor() as u16
}

fn recalc_ev(new_ev: u16, total: u16) -> u16 {
    if total <= 510 && new_ev <= 252 {
        new_ev
    } else if new_ev > 252 {
        252
    } else {
        let excess = total.saturating_sub(510);
        new_ev.saturating_sub(excess)
    }
}

fn recalc_iv(new_iv: u16) -> u16 {
    new_iv.min(31)
}
