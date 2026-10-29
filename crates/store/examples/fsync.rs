//! Measures what `synchronous = FULL` costs on the machine it runs on.
//!
//! ```text
//! cargo run --release --example fsync -- [iterations] [data directory]
//! ```
//!
//! The number that matters is not the mean but the tail: every create is two
//! commits that each wait for an `fsync`, one per file, and every burn is two
//! more. On a local NVMe that is tens of microseconds; on a network-attached
//! volume it can be a thousand times worse, and that difference decides
//! whether one writer connection per file is enough.
//!
//! A temporary directory is used unless one is given, and it is removed on
//! exit. Pass a path on the volume being measured to measure that volume --
//! measuring `/tmp` on a host whose `/tmp` is a RAM disk measures nothing.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use blindnote_store::{FreeSpaceFloor, NewNote, Store, UnixTime};

/// Payload size: a plausible note, well under the 64 KB limit.
const PAYLOAD_BYTES: usize = 4096;

/// How many notes to create and burn when no count is given.
const DEFAULT_ITERATIONS: usize = 200;

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let iterations: usize = args
        .next()
        .map_or(DEFAULT_ITERATIONS, |raw| raw.parse().expect("a count"));
    let given_directory = args.next().map(PathBuf::from);

    // Held for the whole run so the directory outlives the store.
    let scratch = given_directory
        .is_none()
        .then(|| tempfile::tempdir().expect("a temporary directory"));
    let directory = given_directory.unwrap_or_else(|| {
        scratch
            .as_ref()
            .expect("a scratch directory")
            .path()
            .to_path_buf()
    });

    let store = Store::open(&directory, FreeSpaceFloor::NONE)
        .await
        .expect("the store opens");
    let payload = vec![0xAB; PAYLOAD_BYTES];

    let mut creates = Vec::with_capacity(iterations);
    let mut burns = Vec::with_capacity(iterations);
    let mut ids = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let now = UnixTime::now();
        let note = NewNote {
            client_ciphertext: payload.clone(),
            expires_at: now.plus_seconds(86_400),
            views_left: 1,
        };

        let started = Instant::now();
        let id = store.create(note).await.expect("stored");
        creates.push(started.elapsed());
        ids.push(id);
    }

    for id in ids {
        let now = UnixTime::now();
        let started = Instant::now();
        store.burn(id, now).await.expect("burned");
        burns.push(started.elapsed());
    }

    println!("directory:   {}", directory.display());
    println!("payload:     {PAYLOAD_BYTES} bytes");
    println!("iterations:  {iterations}");
    println!();
    report("create (2 commits: key, then note)", &mut creates);
    report("burn   (2 commits + a WAL truncate)", &mut burns);
}

/// Prints the distribution in microseconds, tail first.
fn report(label: &str, samples: &mut [Duration]) {
    samples.sort_unstable();

    println!("{label}");
    for (name, value) in [
        ("min", percentile(samples, 0)),
        ("p50", percentile(samples, 50)),
        ("p90", percentile(samples, 90)),
        ("p99", percentile(samples, 99)),
        ("max", percentile(samples, 100)),
    ] {
        println!("  {name}  {:>9} us", value.as_micros());
    }
    println!();
}

/// The sample at `which` percent of a sorted slice.
fn percentile(sorted: &[Duration], which: usize) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }

    let last = sorted.len() - 1;
    let index = (last * which) / 100;
    sorted[index]
}
