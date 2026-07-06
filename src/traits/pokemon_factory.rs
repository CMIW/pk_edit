//! The [`PokemonFactory`] trait for creating new Pokémon from species data.

use crate::common::types::TrainerID;
use crate::error::PokemonError;
use crate::traits::pokemon::Pokemon;

/// Creates a new Pokémon of a given species with original-trainer data.
///
/// Each generation implements this with its own concrete [`Pokemon`] type as
/// [`Output`](PokemonFactory::Output). The GUI calls this through
/// [`crate::AnyFactory`] so it stays generation-agnostic.
pub trait PokemonFactory {
    /// The concrete Pokémon type produced by this factory.
    type Output: Pokemon;

    /// Creates a new [`Output`](PokemonFactory::Output) for the given species.
    ///
    /// # Errors
    /// Returns [`PokemonError`] if the species name is not found in the database.
    fn gen_pokemon_from_species(
        &self,
        pokemon: &Self::Output,
        species: &str,
        ot_name: &str,
        ot_id: TrainerID,
    ) -> Result<Self::Output, PokemonError>;
}
