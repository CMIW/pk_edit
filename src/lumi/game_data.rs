use crate::common::item_flavor_text_for;
use crate::common::types::{Abilities, Evolution, Pocket};
use crate::traits::game_data::GameData;
use rusqlite::{Connection, Result};

#[derive(Debug, Clone, Copy, Default)]
pub struct LumiGameData;

impl GameData for LumiGameData {
    type Result<R> = rusqlite::Result<R>;

    fn held_items(&self) -> Result<Vec<String>> {
        let conn = Connection::open("pk_edit.db")?;
        let mut stmt =
            conn.prepare("SELECT name_en FROM items WHERE game_family = 'lumi' AND holdable = 1")?;
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

    fn nat_dex_num(&self, species: &str) -> Result<u16> {
        let conn = Connection::open("pk_edit.db")?;
        let res = conn.query_row(
            "SELECT dex_num FROM species WHERE name_en LIKE ?1 AND game_family = 'lumi'",
            [species],
            |row| row.get(0),
        );
        let _ = conn.close();
        res
    }

    fn growth_rate(&self, dex_num: u16) -> Result<String> {
        let conn = Connection::open("pk_edit.db")?;
        let res = conn.query_row(
            "SELECT growth_rate FROM species WHERE dex_num = ?1 AND form = 0 AND game_family = 'lumi'",
            [dex_num],
            |row| row.get(0),
        );
        let _ = conn.close();
        res
    }

    fn pk_species(&self, dex_num: u16) -> Result<String> {
        let conn = Connection::open("pk_edit.db")?;
        let res = conn.query_row(
            "SELECT name_en FROM species WHERE dex_num = ?1 AND form = 0 AND game_family = 'lumi'",
            [dex_num],
            |row| row.get(0),
        );
        let _ = conn.close();
        res
    }

    fn move_data(&self, id: usize) -> Result<(String, String, u8)> {
        let conn = Connection::open("pk_edit.db")?;
        let res = conn.query_row(
            "SELECT COALESCE(t.name_en, 'Normal'), m.name_en, m.pp
             FROM moves m
             LEFT JOIN types t ON m.type = t.id AND t.game_family = 'lumi'
             WHERE m.id_in_game = ?1 AND m.game_family = 'lumi'",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );
        let _ = conn.close();
        res
    }

    fn typing(&self, dex_num: u16) -> Result<(String, Option<String>)> {
        self.typing_form(dex_num, 0)
    }

    fn typing_form(&self, dex_num: u16, form: u8) -> Result<(String, Option<String>)> {
        let conn = Connection::open("pk_edit.db")?;
        let res = conn.query_row(
            "SELECT t1.name_en, t2.name_en
            FROM species s
            JOIN types t1 ON s.type1 = t1.id AND t1.game_family = 'lumi'
            LEFT JOIN types t2 ON s.type2 = t2.id AND t2.game_family = 'lumi'
            WHERE s.dex_num = ?1 AND s.form = ?2 AND s.game_family = 'lumi'",
            rusqlite::params![dex_num, form],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );
        let _ = conn.close();
        res
    }

    fn gender_ratio(&self, dex_num: u16) -> Result<u8> {
        let conn = Connection::open("pk_edit.db")?;
        let ratio: u8 = conn.query_row(
            "SELECT gender_ratio FROM species WHERE dex_num = ?1 AND form = 0 AND game_family = 'lumi'",
            [dex_num],
            |row| row.get(0),
        )?;
        let _ = conn.close();
        Ok(ratio)
    }

    fn abilities(&self, dex_num: u16) -> Self::Result<Abilities> {
        self.abilities_form(dex_num, 0)
    }

    fn abilities_form(&self, dex_num: u16, form: u8) -> Self::Result<Abilities> {
        let conn = Connection::open("pk_edit.db")?;
        let res = conn.query_row(
            "SELECT a1.name_en, a2.name_en, ah.name_en
             FROM species s
             JOIN abilities a1 ON s.ability1 = a1.id_in_game AND a1.game_family = 'lumi'
             LEFT JOIN abilities a2 ON s.ability2 = a2.id_in_game AND a2.game_family = 'lumi'
             LEFT JOIN abilities ah ON s.hidden_ability = ah.id_in_game AND ah.game_family = 'lumi'
             WHERE s.dex_num = ?1 AND s.form = ?2 AND s.game_family = 'lumi'",
            rusqlite::params![dex_num, form],
            |row| {
                Ok(Abilities {
                    slot1: row.get(0)?,
                    slot2: row.get(1)?,
                    hidden: row.get(2)?,
                })
            },
        );
        let _ = conn.close();
        res
    }

    fn ability_ids(&self, dex_num: u16) -> Self::Result<(u16, Option<u16>, Option<u16>)> {
        self.ability_ids_form(dex_num, 0)
    }

    fn ability_ids_form(
        &self,
        dex_num: u16,
        form: u8,
    ) -> Self::Result<(u16, Option<u16>, Option<u16>)> {
        let conn = Connection::open("pk_edit.db")?;
        let res = conn.query_row(
            "SELECT ability1, ability2, hidden_ability FROM species
             WHERE dex_num = ?1 AND form = ?2 AND game_family = 'lumi'",
            rusqlite::params![dex_num, form],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );
        let _ = conn.close();
        res
    }

    fn species(&self) -> Result<Vec<String>> {
        let conn = Connection::open("pk_edit.db")?;
        let mut stmt = conn.prepare(
            "SELECT name_en FROM species WHERE game_family = 'lumi' AND form = 0 ORDER BY dex_num",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut res = Vec::new();
        for result in rows {
            res.push(result?);
        }
        stmt.finalize()?;
        let _ = conn.close();
        Ok(res)
    }

    fn moves(&self) -> Result<Vec<String>> {
        let conn = Connection::open("pk_edit.db")?;
        let mut stmt = conn
            .prepare("SELECT name_en FROM moves WHERE game_family = 'lumi' ORDER BY id_in_game")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut res = Vec::new();
        for result in rows {
            res.push(result?);
        }
        stmt.finalize()?;
        let _ = conn.close();
        Ok(res)
    }

    fn base_stats(&self, dex_num: &u16) -> Result<(u16, u16, u16, u16, u16, u16)> {
        self.base_stats_form(*dex_num, 0)
    }

    fn base_stats_form(&self, dex_num: u16, form: u8) -> Result<(u16, u16, u16, u16, u16, u16)> {
        let conn = Connection::open("pk_edit.db")?;
        let res = conn.query_row(
            "SELECT hp, atk, def, spa, spd, spe FROM species
             WHERE dex_num = ?1 AND form = ?2 AND game_family = 'lumi'",
            rusqlite::params![dex_num, form],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        );
        let _ = conn.close();
        res
    }

    fn evolution(&self, dex_num: &u16) -> Self::Result<Evolution> {
        let conn = Connection::open("pk_edit.db")?;
        let res = conn.query_row(
            "SELECT evolves_into, method, condition FROM evolutions WHERE dex_num = ?1 AND game_family = 'lumi'",
            [dex_num],
            |row| {
                Ok(Evolution {
                    evolves_into: row.get(0)?,
                    method: row.get(1)?,
                    condition: row.get(2)?,
                })
            },
        );
        let _ = conn.close();
        res
    }

    fn items_in_pocket(&self, pocket: Pocket) -> Self::Result<Vec<String>> {
        let pocket_str = pocket.as_db_str();
        let conn = Connection::open("pk_edit.db")?;
        let mut stmt = conn.prepare(
            "SELECT name_en FROM items WHERE game_family = 'lumi' AND pocket = ?1 ORDER BY id_in_game",
        )?;
        let rows = stmt.query_map([pocket_str], |row| row.get(0))?;
        let mut res = Vec::new();
        for result in rows {
            res.push(result?);
        }
        stmt.finalize()?;
        let _ = conn.close();
        Ok(res)
    }

    fn item_flavor_text(&self, name: &str) -> String {
        item_flavor_text_for("lumi", name)
    }

    fn balls_id(&self) -> Result<Vec<u16>> {
        let conn = Connection::open("pk_edit.db")?;
        let mut stmt = conn.prepare(
            "SELECT id_in_game FROM items WHERE game_family = 'lumi' AND pocket = 'balls'",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut res = Vec::new();
        for result in rows {
            res.push(result?);
        }
        stmt.finalize()?;
        let _ = conn.close();
        Ok(res)
    }

    fn item_id_by_name(&self, name: &str) -> Result<usize> {
        let conn = Connection::open("pk_edit.db")?;
        let res = conn.query_row(
            "SELECT id_in_game FROM items WHERE name_en = ?1 AND game_family = 'lumi'",
            [name],
            |row| row.get(0),
        );
        let _ = conn.close();
        res
    }

    fn item_sprite_id(&self, name: &str) -> Result<usize> {
        let conn = Connection::open("pk_edit.db")?;
        let res = conn.query_row(
            "SELECT sprite_id FROM items WHERE name_en = ?1 AND game_family = 'lumi'",
            [name],
            |row| row.get(0),
        );
        let _ = conn.close();
        res
    }

    fn balls_sprite_ids(&self) -> Result<Vec<u16>> {
        let conn = Connection::open("pk_edit.db")?;
        let mut stmt = conn.prepare(
            "SELECT sprite_id FROM items WHERE game_family = 'lumi' AND pocket = 'balls' AND sprite_id IS NOT NULL",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut res = Vec::new();
        for result in rows {
            res.push(result?);
        }
        stmt.finalize()?;
        let _ = conn.close();
        Ok(res)
    }

    fn lowest_level(&self, dex_num: u16) -> u8 {
        let conn = match Connection::open("pk_edit.db") {
            Ok(c) => c,
            Err(_) => return 1,
        };
        let res = conn.query_row(
            "SELECT condition FROM evolutions WHERE evolves_into = ?1 AND game_family = 'lumi' AND method = 'Level'",
            [dex_num],
            |row| row.get::<_, Option<String>>(0),
        );
        let _ = conn.close();
        res.ok()
            .flatten()
            .and_then(|c| c.parse::<u8>().ok())
            .unwrap_or(1)
    }
}
