//! A small HyperLogLog sketch, for approximate daily unique-visitor counts.
//!
//! This is the one place the service forms any view of "how many different
//! people", and it is built so that it cannot become more than a count. A
//! register holds the longest run of leading zero bits ever seen in a value
//! that hashed into its bucket -- never the value, never anything derived from
//! one input alone -- so the sketch cannot be enumerated, matched or reversed
//! back to who was counted. The hashed value is a per-day random salt followed
//! by a client address; the salt rotates every day and is thrown away with it,
//! and the address itself is never stored, only its effect on one 6-bit
//! register. Two sketches of the same traffic with different salts share no
//! structure, so even the sketch bytes carry nothing stable about a visitor.
//!
//! The estimate is approximate on purpose: with `M` registers the standard
//! error is about `1.04 / sqrt(M)`, which at `M = 4096` is ~1.6%. For a count
//! of visitors that is plenty, and the inaccuracy is itself a small privacy
//! margin.

use sha2::{Digest, Sha256};

/// log2 of the register count. 12 gives 4096 registers: ~1.6% standard error
/// and 4 KiB of state, independent of how many visitors are folded in.
const P: u32 = 12;

/// Number of registers.
const M: usize = 1 << P;

/// A fixed-size HyperLogLog sketch.
#[derive(Clone, Debug)]
pub(crate) struct Hll {
    /// One byte per register, each the largest `rho` observed for it.
    registers: Vec<u8>,
}

impl Default for Hll {
    fn default() -> Self {
        Self::new()
    }
}

impl Hll {
    /// An empty sketch.
    pub(crate) fn new() -> Self {
        Self {
            registers: vec![0_u8; M],
        }
    }

    /// Rebuilds a sketch from persisted register bytes.
    ///
    /// A wrong length means the parameter changed between builds; rather than
    /// mix incompatible sketches, the day starts over. Losing one day's
    /// in-progress estimate to a parameter change is the harmless outcome.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() == M {
            Self {
                registers: bytes.to_vec(),
            }
        } else {
            Self::new()
        }
    }

    /// The register bytes, for persisting.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.registers
    }

    /// Folds one observation in: hash `salt || item`, let the top `P` bits
    /// choose a register, and keep the longest leading-zero run of the rest.
    pub(crate) fn add(&mut self, salt: &[u8], item: &[u8]) {
        let mut hasher = Sha256::new();
        hasher.update(salt);
        hasher.update(item);
        let digest = hasher.finalize();

        let hash = u64::from_be_bytes(
            digest[..8]
                .try_into()
                .expect("a SHA-256 digest is 32 bytes, so its first 8 exist"),
        );

        // Top P bits pick the bucket; the value is below M = 4096, so this
        // conversion never actually fails.
        let index = usize::try_from(hash >> (64 - P)).unwrap_or(0);

        // The remaining 64-P bits give rho = (position of the first 1) + 1. A
        // guard bit set inside the shifted-out region caps rho at 64-P+1 when
        // those bits are all zero, so the register never overflows a u8.
        let rest = (hash << P) | (1_u64 << (P - 1));
        #[expect(
            clippy::cast_possible_truncation,
            reason = "rho is bounded by 64 - P + 1 = 53, well within u8"
        )]
        let rho = (rest.leading_zeros() + 1) as u8;

        if rho > self.registers[index] {
            self.registers[index] = rho;
        }
    }

    /// The estimated number of distinct items folded in.
    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "cardinality estimation is inherently floating-point and approximate"
    )]
    pub(crate) fn estimate(&self) -> u64 {
        let m = M as f64;

        let mut harmonic = 0.0_f64;
        let mut empty = 0_usize;
        for &register in &self.registers {
            harmonic += 2.0_f64.powi(-i32::from(register));
            if register == 0 {
                empty += 1;
            }
        }

        // alpha_m for m >= 128 (Flajolet et al.).
        let alpha = 0.7213 / (1.0 + 1.079 / m);
        let raw = alpha * m * m / harmonic;

        // Linear counting is more accurate while the sketch is sparse, which is
        // exactly the small-traffic range this service starts in.
        let estimate = if raw <= 2.5 * m && empty > 0 {
            m * (m / empty as f64).ln()
        } else {
            raw
        };

        estimate.round().max(0.0) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::{Hll, M};

    /// Folds `n` distinct items in and returns the estimate.
    fn estimate_of(n: u64) -> u64 {
        let mut hll = Hll::new();
        let salt = b"test-salt";
        for i in 0..n {
            hll.add(salt, &i.to_be_bytes());
        }
        hll.estimate()
    }

    /// The estimate is within the sketch's error band across the range the
    /// service will actually see. 10% is generous against a ~1.6% standard
    /// error and catches a broken estimator without being flaky.
    #[expect(
        clippy::cast_precision_loss,
        reason = "the tolerance band is computed in f64 against an approximate estimate"
    )]
    #[test]
    fn the_estimate_tracks_the_true_count() {
        for &n in &[0_u64, 1, 10, 100, 1_000, 10_000, 100_000] {
            let got = estimate_of(n);
            let tolerance = (n as f64 * 0.10).max(1.0);
            let low = (n as f64 - tolerance).max(0.0);
            let high = n as f64 + tolerance;
            assert!(
                (got as f64) >= low && (got as f64) <= high,
                "n={n}: estimate {got} outside [{low}, {high}]"
            );
        }
    }

    /// Adding the same item many times counts once.
    #[test]
    fn repeats_do_not_inflate_the_count() {
        let mut hll = Hll::new();
        for _ in 0..10_000 {
            hll.add(b"salt", b"the-same-visitor");
        }
        assert!(hll.estimate() <= 2, "one distinct item should read as ~1");
    }

    /// A different salt produces different registers for the same inputs, so a
    /// sketch carries nothing stable about who was counted across days.
    #[test]
    fn the_salt_changes_the_sketch() {
        let mut a = Hll::new();
        let mut b = Hll::new();
        for i in 0..1_000_u64 {
            a.add(b"monday-salt", &i.to_be_bytes());
            b.add(b"tuesday-salt", &i.to_be_bytes());
        }
        assert_ne!(a.as_bytes(), b.as_bytes());
    }

    /// Round-tripping through bytes preserves the sketch; a wrong length resets.
    #[test]
    fn from_bytes_restores_or_resets() {
        let mut hll = Hll::new();
        for i in 0..500_u64 {
            hll.add(b"salt", &i.to_be_bytes());
        }
        let restored = Hll::from_bytes(hll.as_bytes());
        assert_eq!(restored.as_bytes(), hll.as_bytes());
        assert_eq!(restored.estimate(), hll.estimate());

        let reset = Hll::from_bytes(&[0_u8; 3]);
        assert_eq!(reset.as_bytes().len(), M);
        assert_eq!(reset.estimate(), 0);
    }
}
