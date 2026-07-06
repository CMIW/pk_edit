# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-07-06

### Added

- `GameData::lowest_level()` — queries evolution database to find minimum level for a species
- `Section::validate_checksum()` — validates 16-bit checksum on save file sections
- `SectionID::Display` and `SectionID::to_string()` implementations
- `DetectError::ChecksumFailed` — wraps `SaveDataError` when section checksums are invalid
- `PokemonError::InvalidIndex` — returned for out-of-bounds PC box/slot access
- `SaveDataError::ChecksumMismatch`, `SectionNotFound`, `InvalidOffset` variants
- Evolution data imported into `pk_edit.db` via `seed_db` binary (supports `gen3`, `gen4`, `bdsp`, `lumi` game families)
- Implementation plan for BDSP / Luminescent Platinum support (`docs/bdsp_implementation_plan.md`)

### Changed

- **Breaking**: `Pokemon::lowest_level()` removed from trait; use `GameData::lowest_level(dex_num)` instead
- **Breaking**: `PokemonFactory::gen_pokemon_from_species()` now takes `&Self::Output` (existing Pokémon slot) as first argument — preserves the slot's offset and context
- **Breaking**: `open()` now validates Gen III section checksums; returns `DetectError::ChecksumFailed` on mismatch
- **Breaking**: `SaveFile::new()` returns `Result<Self, SaveDataError>` — validates checksums on construction
- **Breaking**: `Section::data()`, `data_mut()`, `id()`, `save_index()` return `Result` instead of panicking via `expect`/indexing
- **Breaking**: `SaveFile::ot_name()`, `ot_id()`, `game_code()`, `get_party()`, `pocket()`, `save_pocket()`, `trainer()`, `save_trainer()` return `Result` instead of panicking
- `update_iv()`/`update_ev()` now write back to the underlying bitfield/EV struct (previously only updated the cached `Stats` copies)
- `Gen3GameData::gender_ratio()` returns human-readable string (e.g. `"50.0% female"`, `"Always male"`, `"Genderless"`) instead of raw `u8`
- `PCBuffer::new()` returns `Result` instead of silently panicking
- `seed_db` binary expanded: seeds `evolutions` table from Veekun CSV data

### Fixed

- `Pokemon::lowest_level()` implementation removed from `Gen3Pokemon` — now delegated to `GameData::lowest_level()` querying the evolutions table, which correctly returns the evolution trigger level instead of the stored `condition` string directly
- Many `expect()`/`unwrap()` call-sites in `gen3/save/` replaced with proper `Result` propagation

## [0.4.0] - 2026-04-04

### Added

- `open()` entry point: detects generation from raw save file bytes and returns `OpenSave`
- `OpenSave` enum with generation-dispatch methods: `party()`, `pc_box()`, `save_pokemon()`, `swap_pokemon()`, `pocket()`, `save_pocket()`, `trainer_name()`, `trainer_id()`, `game_data()`, `pokemon_factory()`
- `AnyPokemon` generation-dispatch wrapper implementing the `Pokemon` trait
- `AnyGameData` generation-dispatch wrapper implementing the `GameData` trait
- `AnyFactory` generation-dispatch wrapper implementing the `PokemonFactory` trait
- Generation III save file support (Ruby, Sapphire, Emerald, FireRed, LeafGreen)
- `Pokemon` trait covering level, nature, gender, species, OT info, nickname, held item, moves, ability, friendship, Pokérus, IVs, EVs, EXP, shiny/egg/bad-egg status, computed stats, Pokéball, and personality value
- `GameData` trait for static game data queries: species, moves, items, base stats, typing, gender ratio, growth rate, evolution, abilities, bag pockets, and Pokéball sprite IDs
- `PokemonFactory` trait and `Gen3Factory` for generating new Pokémon from a species name
- Trait modules: `bag`, `game_data`, `hyper_training`, `pc`, `pokemon`, `pokemon_factory`, `save_file`
- `Gen3Pokemon` with full encrypted substructure decode/encode and checksum management
- `Gen3GameData` backed by a bundled SQLite database (`pk_edit.db`)
- Generation III charset encoder/decoder (`common::charset::gen3`)
- Common types: `StatBlock`, `ComputedStats`, `TrainerID`, `Gender`, `Pokerus`, `Pocket`, `Move`, `Abilities`, `Evolution`
- `DetectError` and `SaveDataError` error types with `thiserror`-derived implementations
- `seed_db` binary for seeding the bundled SQLite database from CSV sources

### Changed

- Restructured from flat `pokemon/` and `save/` modules into a generation-namespaced layout (`gen3/pokemon/`, `gen3/save/`) with shared `traits/` and `common/` modules
- Migrated static game data storage from embedded arrays to a SQLite database

[0.5.0]: https://github.com/CMIW/pk_edit/releases/tag/v0.5.0
[0.4.0]: https://github.com/CMIW/pk_edit/releases/tag/v0.4.0
