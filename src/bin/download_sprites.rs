//! Download SV-style Pokémon sprites from Project Pokémon into the editor's
//! asset directory.
//!
//! # Usage
//!
//! ```text
//! cargo run --bin download_sprites
//! cargo run --bin download_sprites -- --db /path/to/pk_edit.db --output /path/to/assets/Pokemon
//! ```
//!
//! Sprites are named `{dex:04}.png` for the base form and `{dex:04}_{form:02}.png`
//! for alternate forms, matching the PB8 form byte directly.
//!
//! The set of `(dex, form)` pairs to fetch is read from the `species` table of
//! `pk_edit.db`, so only forms the editor actually knows about are downloaded.
//! Files already present in the output directory are skipped, making the command
//! idempotent and safe to re-run after re-seeding.
//!
//! Two upstream sources are tried in order. If neither returns HTTP 200 the
//! sprite is recorded as missing.

use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{bail, Context, Result};
use rusqlite::Connection;

const URL_PRIMARY: &str = "https://projectpokemon.org/images/sprites-models/sv-sprites-home";
const URL_FALLBACK: &str =
    "https://projectpokemon.org/images/sprites-models/sv-sprites-regular";
const NUM_THREADS: usize = 8;

struct Args {
    db: PathBuf,
    output: PathBuf,
}

fn parse_args() -> Result<Args> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut db: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;

    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--db" => db = it.next().map(PathBuf::from),
            "--output" => output = it.next().map(PathBuf::from),
            other => bail!("unknown argument: {other}"),
        }
    }

    Ok(Args {
        db: db.unwrap_or_else(|| manifest.join("pk_edit.db")),
        output: output.unwrap_or_else(|| manifest.join("../../assets/Pokemon")),
    })
}

/// Reads every distinct `(dex_num, form)` pair from the `species` table.
fn collect_forms(db: &Path) -> Result<BTreeSet<(u16, u8)>> {
    let conn = Connection::open(db).with_context(|| format!("opening {}", db.display()))?;
    let mut stmt =
        conn.prepare("SELECT DISTINCT dex_num, form FROM species ORDER BY dex_num, form")?;
    let rows = stmt.query_map([], |row| {
        let dex: u16 = row.get(0)?;
        let form: u8 = row.get(1)?;
        Ok((dex, form))
    })?;

    let mut set = BTreeSet::new();
    for row in rows {
        set.insert(row?);
    }
    Ok(set)
}

fn sprite_name(dex: u16, form: u8) -> String {
    if form == 0 {
        format!("{dex:04}.png")
    } else {
        format!("{dex:04}_{form:02}.png")
    }
}

/// Try to fetch from `url`. Returns `Some(bytes)` on HTTP 200, `None` on
/// any error or non-200 status.
fn try_fetch(url: &str) -> Option<Vec<u8>> {
    let resp = ureq::get(url).call().ok()?;
    if resp.status() != 200 {
        return None;
    }
    let mut buf = Vec::new();
    resp.into_reader()
        .take(10 * 1024 * 1024)
        .read_to_end(&mut buf)
        .ok()?;
    Some(buf)
}

/// Try primary URL, fall back to secondary on non-200.
fn fetch_sprite(name: &str) -> Option<Vec<u8>> {
    let primary = format!("{URL_PRIMARY}/{name}");
    if let Some(bytes) = try_fetch(&primary) {
        return Some(bytes);
    }
    let fallback = format!("{URL_FALLBACK}/{name}");
    try_fetch(&fallback)
}

struct Stats {
    downloaded: u32,
    missing: u32,
}

fn main() -> Result<()> {
    let args = parse_args()?;

    if !args.output.is_dir() {
        std::fs::create_dir_all(&args.output)
            .with_context(|| format!("creating output dir {}", args.output.display()))?;
    }

    let forms = collect_forms(&args.db)?;
    let total = forms.len();
    println!("{total} sprite(s) referenced by the database.");

    // Partition into skip vs fetch lists.
    let mut to_fetch: VecDeque<String> = VecDeque::new();
    let mut skipped = 0u32;
    for (dex, form) in forms {
        let name = sprite_name(dex, form);
        if args.output.join(&name).exists() {
            skipped += 1;
        } else {
            to_fetch.push_back(name);
        }
    }
    let to_download = to_fetch.len();
    if skipped > 0 {
        println!("{skipped} already present, downloading {to_download} ...");
    }

    let stats = Arc::new(Mutex::new(Stats {
        downloaded: 0,
        missing: 0,
    }));

    // Split work into NUM_THREADS chunks via round-robin.
    let mut buckets: Vec<VecDeque<String>> = (0..NUM_THREADS).map(|_| VecDeque::new()).collect();
    for (i, name) in to_fetch.into_iter().enumerate() {
        buckets[i % NUM_THREADS].push_back(name);
    }

    let output = Arc::new(args.output);

    let mut handles = Vec::with_capacity(NUM_THREADS);
    for work in buckets {
        let stats = Arc::clone(&stats);
        let output = Arc::clone(&output);
        let handle = thread::spawn(move || {
            let mut dl = 0u32;
            let mut miss = 0u32;
            for name in work {
                let result = fetch_sprite(&name);
                match result {
                    Some(bytes) => {
                        let dest = output.join(&name);
                        if let Err(e) = std::fs::write(&dest, &bytes) {
                            eprintln!("  error writing {}: {e}", dest.display());
                        } else {
                            dl += 1;
                        }
                    }
                    None => {
                        miss += 1;
                    }
                }
            }
            let mut s = stats.lock().unwrap();
            s.downloaded += dl;
            s.missing += miss;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap_or_else(|e| eprintln!("worker panicked: {e:?}"));
    }

    let s = stats.lock().unwrap();
    println!(
        "Done. {} downloaded, {skipped} already present, {} not available upstream.",
        s.downloaded, s.missing
    );
    Ok(())
}
