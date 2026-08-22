![GitHub repo size](https://img.shields.io/github/repo-size/CMIW/pk_edit)
![GitHub contributors](https://img.shields.io/github/contributors/CMIW/pk_edit)
![GitHub stars](https://img.shields.io/github/stars/CMIW/pk_edit?style=social)
![GitHub forks](https://img.shields.io/github/forks/CMIW/pk_edit?style=social)

# pk_edit

Multi-generation Pokémon save file editing library.

Provides utilities to read, modify, and write save files from Pokémon games through a
generation-agnostic trait system. Currently supports **Generation III** (Ruby, Sapphire,
Emerald, FireRed, LeafGreen) and **Generation VIII** (Brilliant Diamond, Shining Pearl,
Luminescent Platinum).

This is the backend for the save file editor [pk_editor](https://github.com/CMIW/pk_editor).

## Architecture

```
src/
├── lib.rs              # Public API, AnyPokemon/AnyGameData/AnyFactory dispatch enums,
│                         open() entry point, save file detection
├── common/
│   ├── types.rs        # Shared types: TrainerID, Gender, StatBlock, Move, Abilities, etc.
│   ├── inventory8b.rs  # BDSP/Lumi item pocket read/write (Gen 8b format)
│   └── charset/        # Character encoding tables
├── traits/
│   ├── pokemon.rs      # Pokemon trait (name, ability, stats, moves, forms, ribbons, …)
│   ├── game_data.rs    # GameData trait (SQLite-backed queries for species, abilities, moves, items)
│   ├── pc.rs           # PC box storage trait
│   └── pokemon_factory.rs  # PokemonFactory trait (new Pokémon from species)
├── gen3/               # Generation III implementation
│   ├── game_data.rs
│   ├── pokemon/        # Gen3Pokemon, Gen3Factory, crypto
│   └── save/           # SaveFile, sections, trainer, PC, storage
├── bdsp/               # Brilliant Diamond / Shining Pearl implementation
│   ├── game_data.rs
│   ├── pokemon/        # BdspPokemon, BdspFactory, crypto, personal info
│   └── save/           # SaveFile, trainer, PC
├── lumi/               # Luminescent Platinum implementation
│   ├── game_data.rs
│   ├── pokemon/        # LumiPokemon, LumiFactory, crypto, personal info
│   └── save/           # SaveFile, trainer, PC
├── bin/
│   ├── seed_db.rs      # Database seeder (PKLumiHex personal info + Veekun CSV data)
│   └── download_sprites.rs  # Parallel sprite downloader with fallback URLs
├── error.rs            # PokemonError, SaveDataError, DetectError
└── misc.rs             # SQLite database extraction, embedded DB, constants
```

## Supported Formats

| Generation | Games | Status |
|---|---|---|
| III | Ruby, Sapphire, Emerald, FireRed, LeafGreen | ✅ Full support |
| VIII | Brilliant Diamond, Shining Pearl | ✅ Full support |
| VIII | Luminescent Platinum (ROM hack) | ✅ Full support |

## Features

- **Generation-agnostic traits** — `Pokemon`, `GameData`, `PokemonFactory` traits let the
  GUI and CLI work across all supported formats without per-generation branching.
- **SQLite-backed game data** — species stats, abilities, types, moves, items, and evolutions
  are queried from an embedded `pk_edit.db` seeded from PKHeX personal info binaries and
  Veekun CSV data.
- **Form support** — alternate forms (Alolan, Galarian, Mega, Unown letters, etc.) are stored
  in the DB and handled via `_form` suffix methods. BDSP/Lumi read/write form values directly
  from the save data.
- **Sprite lookup** — per-form sprite resolution with fallback to base form and a transparent
  placeholder.
- **Pokédex** — Gen3 uses section-based bitfields; BDSP uses u32-per-species; Lumi uses
  nibble-packed states covering Gen 1–9 (1025 species).
- **Save-as** — full round-trip read/write with MD5 integrity hash recomputation (BDSP/Lumi).

## Tools

```sh
# Seed the SQLite database from PKLumiHex + Veekun data
cargo run --bin seed_db -- \
    --pkhex  /path/to/PKLumiHex \
    --veekun /path/to/pokedex \
    --output pk_edit.db

# Download Pokémon sprites (parallel, with fallback URLs)
cargo run --bin download_sprites
```

## Testing

```sh
cargo test
```

## Documentation

```sh
cargo doc --open
```

## License
Licensed under [0BSD](LICENSE).
