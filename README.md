![GitHub repo size](https://img.shields.io/github/repo-size/CMIW/pk_edit)
![GitHub contributors](https://img.shields.io/github/contributors/CMIW/pk_edit)
![GitHub stars](https://img.shields.io/github/stars/CMIW/pk_edit?style=social)
![GitHub forks](https://img.shields.io/github/forks/CMIW/pk_edit?style=social)

# pk_edit

Multi-generation Pokémon save file editing library.

Provides utilities to read, modify, and write save files from Pokémon games through a
generation-agnostic trait system. Currently supports **Generation III** (Ruby, Sapphire,
Emerald, FireRed, LeafGreen) with **Generation VIII BDSP / Luminescent Platinum** planned.

This is the backend for the save file editor [pk_editor](https://github.com/CMIW/pk_editor).

## Architecture

```
src/
├── lib.rs            # Public API, AnyPokemon/AnyGameData/AnyFactory dispatch enums,
│                        open() entry point, save file detection
├── common/
│   └── types.rs      # Shared types: TrainerID, Gender, StatBlock, Move, etc.
├── traits/
│   ├── pokemon.rs    # Pokemon trait (34 methods)
│   ├── game_data.rs  # GameData trait (SQLite-backed queries)
│   ├── save_file.rs  # SaveFile trait
│   └── pokemon_factory.rs  # PokemonFactory trait
├── gen3/             # Generation III implementation
│   ├── game_data.rs
│   ├── pokemon/      # Gen3Pokemon, Gen3Factory, crypto
│   └── save/         # SaveFile, sections, trainer, PC, storage
├── error.rs          # PokemonError, SaveDataError, DetectError
└── misc.rs           # SQLite database extraction
```

## Supported Formats

| Generation | Games | Status |
|---|---|---|
| III | Ruby, Sapphire, Emerald, FireRed, LeafGreen | ✅ Full support |
| VIII | Brilliant Diamond, Shining Pearl | 🔄 Planned |
| VIII | Luminescent Platinum (ROM hack) | 🔄 Planned |

## Documentation
```sh
cargo doc --open
```
