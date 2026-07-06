# BDSP & Luminescent Platinum Implementation Plan

## 1. Module Structure

Two completely independent, self-contained modules with full code duplication:

```
core/pk_edit/src/
├── lib.rs
├── bdsp/
│   ├── mod.rs
│   ├── game_data.rs
│   └── pokemon/
│   │   ├── mod.rs
│   │   ├── pokemon.rs
│   │   ├── data.rs
│   │   ├── crypto.rs
│   │   └── factory.rs
│   └── save/
│       ├── mod.rs
│       ├── trainer.rs
│       └── pc.rs
├── lumi/
│   ├── mod.rs
│   ├── game_data.rs
│   └── pokemon/
│   │   ├── mod.rs
│   │   ├── pokemon.rs
│   │   ├── data.rs
│   │   ├── crypto.rs
│   │   └── factory.rs
│   └── save/
│       ├── mod.rs
│       ├── trainer.rs
│       └── pc.rs
```

## 2. Constants

### Save File Sizes

| Variant | Size | Name |
|---------|------|------|
| BDSP 1.0 | `0xE9828` (956,456) | |
| BDSP 1.1 | `0xEDC20` (972,832) | |
| BDSP 1.2 | `0xEED8C` (978,316) | |
| BDSP 1.3 | `0xEF0A4` (979,108) | |
| Lumi 1.1 | `0xEDC20` | same as BDSP |
| Lumi 1.3 | `0xEF0A4` | same as BDSP |

### Pokémon Data

| Property | Value |
|----------|-------|
| `SIZE_STORED` | `0x148` (328 bytes) |
| `SIZE_PARTY` | `0x158` (344 bytes) |
| `SIZE_BLOCK` | `0x50` (80 bytes) |
| `SIZE_BOXSLOT` | `0x158` (same as party) |

### Save Offsets (same for BDSP and Lumi)

| Region | Offset | Size | Notes |
|--------|--------|------|-------|
| Version | `0x00000` | 4 | u32 LE |
| FlagWork | `0x00004` | — | out of scope |
| Items | `0x0563C` | — | out of scope |
| Party | `0x14098` | 6 × `0x158` + 2 | 6 slots + count byte + marking index |
| BoxLayout | `0x148AA` | `0x64A` | names, team slots, wallpaper |
| Box data start | `0x14EF4` | 40 × 30 × `0x158` | 40 boxes, 30 slots each |
| Config | `0x79B74` | `0x40` | language, etc |
| MyStatus | `0x79BB4` | `0x50` | OT, TID/SID, gender, money |
| PlayTime | `0x79C04` | 4 | hours/minutes/seconds |
| Contest | `0x79C08` | `0x720` | out of scope |
| Zukan (dex) | `0x7A328` | `0x30B8` | out of scope — BDSP & Lumi differ |
| BattleTrainer | `0x7D3E0` | `0x1618` | out of scope — BDSP & Lumi differ |
| MD5 hash | last `0x10` | 16 | end of file |

### Box Layout (at `0x148AA`)

| Sub-field | Offset in block | Size |
|-----------|----------------|------|
| Box names | `0x000` | 40 × `0x1A` (26 bytes each, UTF-16LE) |
| Team names | `0x680` | 6 × `0x16` (22 bytes each) |
| Team position | `0x5D4` | 36 × 2 bytes (6 teams × 6 slots, i16 LE) |
| LockedTeam | `0x61C` | 1 byte |
| BoxesUnlocked | `0x61D` | 1 byte |
| CurrentBox | `0x61E` | 1 byte |
| Wallpapers | `0x620` | 40 bytes (1 per box, -1 = default) |
| StatusPut | `0x648` | 2 bytes |

## 3. Pokémon Byte Layout (PB8 — 344 bytes)

### Header + Block A (offsets `0x00`–`0x57`)

| Offset | Size | Field | Type |
|--------|------|-------|------|
| 0x00 | 4 | EncryptionConstant | u32 LE |
| 0x04 | 2 | Sanity | u16 LE (always 0) |
| 0x06 | 2 | Checksum | u16 LE |
| 0x08 | 2 | Species | u16 LE |
| 0x0A | 2 | HeldItem | u16 LE |
| 0x0C | 4 | ID32 (TID16 \| SID16<<16) | u32 LE |
| 0x10 | 4 | EXP | u32 LE |
| 0x14 | 2 | Ability | u16 LE |
| 0x16 | 1 | AbilityNumber (bits 0-2), IsFavorite (bit 3), CanGigantamax (bit 4) | u8 |
| 0x18 | 2 | MarkValue | u16 LE |
| 0x1C | 4 | PID | u32 LE |
| 0x20 | 1 | Nature | u8 |
| 0x21 | 1 | StatNature | u8 |
| 0x22 | 1 | Flags: FatefulEncounter(bit0), Gender(bit2-3) | u8 |
| 0x24 | 2 | Form | u16 LE |
| 0x26 | 1 | EV_HP | u8 |
| 0x27 | 1 | EV_ATK | u8 |
| 0x28 | 1 | EV_DEF | u8 |
| 0x29 | 1 | EV_SPE | u8 |
| 0x2A | 1 | EV_SPA | u8 |
| 0x2B | 1 | EV_SPD | u8 |
| 0x2C | 6 | Contest stats (Cool, Beauty, Cute, Smart, Tough, Sheen) | u8 × 6 |
| 0x32 | 1 | PKRS (upper nibble = strain, lower = days) | u8 |
| 0x34-0x47 | 20 | Ribbon flags (u64 at 0x34, u64 at 0x3C, u64 at 0x44) | bitfields |
| 0x48 | 4 | Sociability | u32 LE |
| 0x50 | 1 | HeightScalar | u8 |
| 0x51 | 1 | WeightScalar | u8 |

### Block B — Nickname & Moves (offsets `0x58`–`0xA7`)

| Offset | Size | Field |
|--------|------|-------|
| 0x58 | 26 | Nickname (UTF-16LE, max 12 chars + null) |
| 0x72 | 2 | Move1 (u16 LE) |
| 0x74 | 2 | Move2 |
| 0x76 | 2 | Move3 |
| 0x78 | 2 | Move4 |
| 0x7A | 1 | Move1_PP |
| 0x7B | 1 | Move2_PP |
| 0x7C | 1 | Move3_PP |
| 0x7D | 1 | Move4_PP |
| 0x7E | 1 | Move1_PPUps |
| 0x7F | 1 | Move2_PPUps |
| 0x80 | 1 | Move3_PPUps |
| 0x81 | 1 | Move4_PPUps |
| 0x82 | 2 | RelearnMove1 |
| 0x84 | 2 | RelearnMove2 |
| 0x86 | 2 | RelearnMove3 |
| 0x88 | 2 | RelearnMove4 |
| 0x8A | 2 | Stat_HPCurrent |
| 0x8C | 4 | IV32: HP(0-4), ATK(5-9), DEF(10-14), SPE(15-19), SPA(20-24), SPD(25-29), IsEgg(30), IsNicknamed(31) |
| 0x90 | 1 | DynamaxLevel |
| 0x94 | 4 | Status_Condition |
| 0x98 | 4 | Palma |

### Block C — Handler (offsets `0xA8`–`0xF7`)

| Offset | Size | Field |
|--------|------|-------|
| 0xA8 | 26 | HT_Name (UTF-16LE) |
| 0xC2 | 1 | HT_Gender |
| 0xC3 | 1 | HT_Language |
| 0xC4 | 1 | CurrentHandler |
| 0xC6 | 2 | HT_TrainerID |
| 0xC8 | 1 | HT_Friendship |
| 0xC9 | 1 | HT_Intensity |
| 0xCA | 1 | HT_Memory |
| 0xCB | 1 | HT_Feeling |
| 0xCC | 2 | HT_TextVar |
| 0xCE | 14 | PokeJob flags |
| 0xDC | 1 | Fullness |
| 0xDD | 1 | Enjoyment |
| 0xDE | 1 | Version (game version byte) |
| 0xDF | 1 | BattleVersion |
| 0xE2 | 1 | Language |
| 0xE4 | 4 | FormArgument |
| 0xE8 | 1 | AffixedRibbon |

### Block D — OT (offsets `0xF8`–`0x147`)

| Offset | Size | Field |
|--------|------|-------|
| 0xF8 | 26 | OT_Name (UTF-16LE) |
| 0x112 | 1 | OT_Friendship |
| 0x113 | 1 | OT_Intensity |
| 0x114 | 1 | OT_Memory |
| 0x116 | 2 | OT_TextVar |
| 0x118 | 1 | OT_Feeling |
| 0x119 | 1 | Egg_Year |
| 0x11A | 1 | Egg_Month |
| 0x11B | 1 | Egg_Day |
| 0x11C | 1 | Met_Year |
| 0x11D | 1 | Met_Month |
| 0x11E | 1 | Met_Day |
| 0x120 | 2 | Egg_Location |
| 0x122 | 2 | Met_Location |
| 0x124 | 1 | Ball |
| 0x125 | 1 | Met_Level (bits 0-6) \| OT_Gender (bit 7) |
| 0x126 | 1 | HyperTrainFlags (6 bits) |
| 0x127 | 14 | RecordFlags (move record flags) |
| 0x135 | 8 | Tracker (HOME tracker, u64 LE) |

### Battle Stats (offsets `0x148`–`0x157`, party-only)

| Offset | Size | Field |
|--------|------|-------|
| 0x148 | 1 | Stat_Level |
| 0x14A | 2 | Stat_HPMax |
| 0x14C | 2 | Stat_ATK |
| 0x14E | 2 | Stat_DEF |
| 0x150 | 2 | Stat_SPE |
| 0x152 | 2 | Stat_SPA |
| 0x154 | 2 | Stat_SPD |

## 4. Crypto — Gen 8 Block Shuffle & XOR

### Checksum Calculation
```rust
fn calculate_checksum(data: &[u8]) -> u16 {
    let mut chk: u16 = 0;
    for i in (8..SIZE_STORED).step_by(2) {
        chk = chk.wrapping_add(u16::from_le_bytes([data[i], data[i+1]]));
    }
    chk
}
```

### Encryption Key Stream
```
seed = EncryptionConstant (u32 from offset 0x00)
for each u16 word in blocks [8..8+4*BLOCK_SIZE]:
    seed = (0x41C64E6D * seed) + 0x00006073
    data_word ^= (seed >> 16) as u16
```

### Block Shuffle
```
sv = (EncryptionConstant >> 13) & 31
```
The shuffle order is determined by `sv` into a 24-entry table. Each entry gives a permutation of `[0, 1, 2, 3]` indicating the source block order.

**Decrypt:**
1. XOR blocks with key stream using `EncryptionConstant` as seed
2. Unshuffle blocks: for output block `i`, copy source block `SHUFFLE_TABLE[sv * 4 + i]`

**Encrypt:**
1. Shuffle blocks (inverse permutation): for output block `i`, copy source block `UNSHUFFLE_TABLE[sv * 4 + i]`
2. XOR blocks with key stream

### Shuffle Tables (from PKLumiHex)
SHUFFLE_TABLE (BlockPosition): `[0,1,2,3, 0,1,3,2, 0,2,1,3, 0,3,1,2, 0,2,3,1, 0,3,2,1, 1,0,2,3, 1,0,3,2, 2,0,1,3, 3,0,1,2, 2,0,3,1, 3,0,2,1, 1,2,0,3, 1,3,0,2, 2,1,0,3, 3,1,0,2, 2,3,0,1, 3,2,0,1, 1,2,3,0, 1,3,2,0, 2,1,3,0, 3,1,2,0, 2,3,1,0, 3,2,1,0]`

UNSHUFFLE_TABLE (BlockPositionInvert): `[0,1,2,4,3,5,6,7,12,18,13,19,8,10,14,20,16,22,9,11,15,21,17,23]`

### Encryption Detection
A PKM is encrypted if `u16 at offset 0x70 != 0 || u16 at offset 0x110 != 0`.

## 5. Save File — MD5 Hash

### Hash Calculation
```
hash_input = data[0..end-0x10] ++ [0; 16] ++ data[end..]
md5_hash = MD5(hash_input)
```
Where `end` = `data.len()`. The hash is stored in the last 16 bytes of the file.

### Validation
Computes MD5 as above and compares with stored hash.

### Write-back
Modify party/box data in buffer, recalculate MD5, write to last 16 bytes.

## 6. String Encoding

- **UTF-16 Little Endian**
- Null-terminated (`0x0000`)
- Max OT name length: 12 characters
- Max nickname length: 12 characters
- Max box name length: 13 characters (26 bytes including null)

Encoding: write `u16` LE for each character, then write `0x0000` terminator.
Decoding: read `u16` LE pairs until `0x0000`.

## 7. Trainer Info (MyStatus8b at offset `0x79BB4`, 0x50 bytes)

| Offset | Size | Field |
|--------|------|-------|
| 0x00 | 4 | ID32 (TID16 | SID16 << 16) |
| 0x04 | 2 | TID16 |
| 0x06 | 2 | SID16 |
| 0x08 | 26 | OT name (UTF-16LE, max 12 chars) |
| 0x22 | 1 | Game (BD=0x2F, SP=0x30) |
| 0x23 | 1 | Gender (1 byte: 0=female, 1=male) |
| 0x24 | 4 | Money (u32 LE) |
| 0x28 | 4 | BP (u32 LE) |

## 8. DB Queries — game_data.rs

Both `BdspGameData` and `LumiGameData` implement the `GameData` trait identically, except for the `game_family` filter:

- `BdspGameData`: all queries use `WHERE game_family = 'bdsp'`
- `LumiGameData`: all queries use `WHERE game_family = 'lumi'`

Both are unit structs.

## 9. Detection in lib.rs

### Constants
```rust
const BDSP_SIZES: &[usize] = &[0xE9828, 0xEDC20, 0xEED8C, 0xEF0A4];
const LUMI_SIZES: &[usize] = &[0xEDC20, 0xEF0A4];
```

### Detection Order

1. If size matches one of `LUMI_SIZES` and `(u32 at offset 0) & 0xFFFF0000 == 0xFFFF0000` → Lumi
2. If size matches one of `BDSP_SIZES` and `(u32 at offset 0)` is 0, 1, 2, or 3 → Bdsp

### Lumi version encoding
```
u32 at offset 0: 0xFFFF0000 | (revision as u16)
storage: WriteUInt32(data, 0xFFFF0000 | value)
```
As used in PKLumiHex: `WriteUInt32LittleEndian(Data.AsSpan(0), (uint)(0xFFFF0000 | value))`

### BDSP version values
| Value | Enum |
|-------|------|
| 0 | Gem8Version.V1_0 |
| 1 | Gem8Version.V1_1 |
| 2 | Gem8Version.V1_2 |
| 3 | Gem8Version.V1_3 |
## 10. Any Enum Changes

### AnyPokemon — new variant
```rust
pub enum AnyPokemon {
    Gen3(Gen3Pokemon),
    Bdsp(BdspPokemon),
    Lumi(LumiPokemon),
}
```

### AnyGameData — new variant
```rust
pub enum AnyGameData {
    Gen3(Gen3GameData),
    Bdsp(BdspGameData),
    Lumi(LumiGameData),
}
```

### OpenSave — new variant
```rust
pub enum OpenSave {
    Gen3(gen3::save::SaveFile),
    Bdsp(bdsp::save::SaveFile),
    Lumi(lumi::save::SaveFile),
}
```

### Delegate Methods (add match arms in all):

| Method | New arms |
|--------|----------|
| `party()` | `Bdsp(s) => s.party().map(\|v\| v.into_iter().map(AnyPokemon::Bdsp).collect())` |
| `pc_box(n)` | same pattern |
| `party_count()` | from save |
| `party_mut()` | from save |
| `save_pokemon(...)` | pattern match and delegate |
| `swap_pokemon(...)` | pattern match and delegate |
| `trainer_name()` | from save |
| `trainer_id()` | from save |
| `raw_data()` | from save |

## 11. Implementation Steps (Execution Order)

### Step 1: bdsp/pokemon/crypto.rs
- Implement `decrypt_array8(data: &[u8]) -> Vec<u8>`
- Implement `encrypt_array8(data: &[u8]) -> Vec<u8>`
- Implement `calculate_checksum(data: &[u8]) -> u16`
- Implement `crypt_pkm(data: &mut [u8], seed: u32)`
- Implement `shuffle_array(data: &[u8], sv: u32) -> Vec<u8>`
- Constants: `SIZE_STORED`, `SIZE_PARTY`, `SIZE_BLOCK`, shuffle tables

### Step 2: bdsp/pokemon/data.rs
- Define `BdspStatBlock { hp, attack, defense, special_attack, special_defense, speed }`
- Define `BdspComputedStats` (same fields)
- Define `BdspMoves { moves: [u16; 4], pp: [u8; 4], pp_ups: [u8; 4] }`
- Define `BdspIVs` or use bitfield parsing

### Step 3: bdsp/pokemon/pokemon.rs
- Define `BdspPokemon` struct with `data: [u8; SIZE_PARTY]` and `offset: usize`
- `new() -> Self`, `from_bytes(data, offset) -> Self`, `to_bytes(&self) -> Vec<u8>`
- Implement `Pokemon` trait (all 34 methods)

### Step 4: bdsp/pokemon/factory.rs
- Define `BdspFactory` unit struct
- Implement `PokemonFactory<Output = BdspPokemon>`

### Step 5: bdsp/save/trainer.rs
- Define `BdspTrainer { ot_name, tid, sid, gender, language, money, version, play_time }`
- Parse from MyStatus at 0x79BB4 and PlayTime at 0x79C04

### Step 6: bdsp/save/pc.rs
- Box slot math: `box_start + (box_number * 30 + slot) * SIZE_PARTY`
- Box name read/write via BoxLayout offsets

### Step 7: bdsp/save/mod.rs
- Define `BdspSaveFile { data: Vec<u8> }`
- `new(data)`: validate length + MD5 hash, store data
- `party()`, `pc_box()`, `save_pokemon()`, `swap_pokemon()`, `trainer()`, `to_bytes()`

### Step 8: bdsp/game_data.rs
- Define `BdspGameData` unit struct
- Implement `GameData` trait with `game_family = 'bdsp'`

### Step 9: bdsp/mod.rs
- Module declarations and re-exports

### Step 10: Repeat Steps 1-9 for lumi/
- Struct names prefixed with `Lumi` instead of `Bdsp`
- `game_family = 'lumi'` in game_data.rs
- Different MaxSpeciesID / MaxItemID constants
- Save detection checks for 0xFFFF version prefix
- Lumi sizes: `0xEDC20`, `0xEF0A4`

### Step 11: Update src/lib.rs
- `pub mod bdsp; pub mod lumi;`
- Add variants to `AnyPokemon`, `AnyGameData`, `AnyFactory`, `OpenSave`
- Add match arms in all delegate methods
- Add detection in `open()` (Lumi first, then BDSP, then Gen3)
- Re-exports

### Step 12: Tests

#### Save load/save roundtrip
- Load real BDSP save file (if available)
- Read party, read PC boxes
- Serialize back
- Compare MD5 hash with original
- Verify byte-for-byte match

#### BDSP vs Lumi detection
- Construct data with version=0 -> detected as BDSP
- Construct data with 0xFFFF prefix -> detected as Lumi
- Wrong sizes -> UnknownFormat

#### Pokemon roundtrip
- Decrypt a known PKM byte array
- Check all fields parse correctly
- Encrypt and decrypt again
- Verify all fields preserved

## 12. Out of Scope (for this PR)

- Bag/inventory
- Daycare
- Underground
- Battle teams
- Poketch, seals, contests, poffins
- Event flags/works
- Zukan/Pokedex
- BattleTrainerStatus
- Any save sub-structure beyond party, boxes, and trainer info
