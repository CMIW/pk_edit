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
    /// Form-aware variant of [`typing`](Self::typing). Form 0 is the base form.
    fn typing_form(&self, dex_num: u16, form: u8) -> Self::Result<(String, Option<String>)>;
    fn gender_ratio(&self, dex_num: u16) -> Self::Result<u8>;
    fn species(&self) -> Self::Result<Vec<String>>;
    fn moves(&self) -> Self::Result<Vec<String>>;
    fn base_stats(&self, dex_num: &u16) -> Self::Result<(u16, u16, u16, u16, u16, u16)>;
    /// Form-aware variant of [`base_stats`](Self::base_stats). Form 0 is base.
    fn base_stats_form(&self, dex_num: u16, form: u8)
        -> Self::Result<(u16, u16, u16, u16, u16, u16)>;
    fn evolution(&self, dex_num: &u16) -> Self::Result<Evolution>;
    fn abilities(&self, dex_num: u16) -> Self::Result<Abilities>;
    /// Form-aware variant of [`abilities`](Self::abilities). Form 0 is base.
    fn abilities_form(&self, dex_num: u16, form: u8) -> Self::Result<Abilities>;
    /// Returns the in-game ability IDs for a species as
    /// `(slot1, slot2, hidden)`.
    ///
    /// Generation VIII stores both the ability *ID* and the ability *slot*
    /// separately, so writers need the ID as well as the slot index.
    fn ability_ids(&self, dex_num: u16) -> Self::Result<(u16, Option<u16>, Option<u16>)>;
    /// Form-aware variant of [`ability_ids`](Self::ability_ids). Form 0 is base.
    fn ability_ids_form(
        &self,
        dex_num: u16,
        form: u8,
    ) -> Self::Result<(u16, Option<u16>, Option<u16>)>;
    fn items_in_pocket(&self, pocket: Pocket) -> Self::Result<Vec<String>>;
    /// Returns the English flavor text for a named item, or an empty string if
    /// the item is unknown or has no description.
    fn item_flavor_text(&self, name: &str) -> String;
    /// Returns the item IDs of all Pokéballs available in this generation.
    fn balls_id(&self) -> Self::Result<Vec<u16>>;
    /// Returns the item ID for a named item, or an error if the name is unknown.
    fn item_id_by_name(&self, name: &str) -> Self::Result<usize>;
    /// Returns the SV sprite ID for a named item (used for image file lookup).
    fn item_sprite_id(&self, name: &str) -> Self::Result<usize>;
    /// Returns the SV sprite IDs of all Pokéballs available in this generation.
    fn balls_sprite_ids(&self) -> Self::Result<Vec<u16>>;
    /// Returns the minimum level a Pokémon of the given species can exist at
    /// (i.e. the level it evolved from its pre-evolution, or 1 if it has no pre-evolution).
    fn lowest_level(&self, dex_num: u16) -> u8;
}
