use crate::common::types::Pocket;
use crate::common::types::{Abilities, Evolution};

pub trait GameData {
    type Result<R>;
    fn held_items(&self) -> Self::Result<Vec<String>>;
    fn nat_dex_num(&self, species: &str) -> Self::Result<u16>;
    fn growth_rate(&self, dex_num: u16) -> Self::Result<String>;
    fn pk_species(&self, dex_num: u16) -> Self::Result<String>;
    fn move_data(&self, id: usize) -> Self::Result<(String, String, u8)>;
    fn typing(&self, dex_num: u16) -> Self::Result<(String, Option<String>)>;
    fn gender_ratio(&self, dex_num: u16) -> Self::Result<String>;
    fn species(&self) -> Self::Result<Vec<String>>;
    fn moves(&self) -> Self::Result<Vec<String>>;
    fn base_stats(&self, dex_num: &u16) -> Self::Result<(u16, u16, u16, u16, u16, u16)>;
    fn evolution(&self, dex_num: &u16) -> Self::Result<Evolution>;
    fn abilities(&self, dex_num: u16) -> Self::Result<Abilities>;
    fn items_in_pocket(&self, pocket: Pocket) -> Self::Result<Vec<String>>;
    /// Returns the item IDs of all Pokéballs available in this generation.
    fn balls_id(&self) -> Self::Result<Vec<u16>>;
    /// Returns the item ID for a named item, or an error if the name is unknown.
    fn item_id_by_name(&self, name: &str) -> Self::Result<usize>;
    /// Returns the SV sprite ID for a named item (used for image file lookup).
    fn item_sprite_id(&self, name: &str) -> Self::Result<usize>;
    /// Returns the SV sprite IDs of all Pokéballs available in this generation.
    fn balls_sprite_ids(&self) -> Self::Result<Vec<u16>>;
}
