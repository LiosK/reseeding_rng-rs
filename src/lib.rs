//! [`ReseedingRng`] that periodically reseeds the underlying pseudorandom number
//! generator.
//!
//! ```rust
//! use rand::{RngExt as _, rngs::StdRng, rngs::SysRng};
//! use reseeding_rng::ReseedingRng;
//!
//! let mut rng = ReseedingRng::<StdRng, _>::try_new(1024 * 64, SysRng)
//!     .expect("couldn't initialize ReseedingRng due to SysRng failure");
//! println!("{:?}", rng.random::<[char; 4]>());
//! ```
//!
//! This crate provides a simplified reimplementation of `ReseedingRng` for use with
//! the random number generators from the `rand` crate v0.10, which no longer
//! includes [the `ReseedingRng` from v0.9] and earlier.
//!
//! This crate is `no_std`-compatible.
//!
//! [the `ReseedingRng` from v0.9]: https://docs.rs/rand/0.9.2/rand/rngs/struct.ReseedingRng.html

#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use core::fmt;
use rand_core::{Rng, SeedableRng, TryCryptoRng, TryRng};

/// A wrapper that periodically reseeds the underlying pseudorandom number generator.
///
/// This type reseeds the underlying generator every time a specified number of random bytes have
/// been produced. If the periodic reseeding attempt fails, `ReseedingRng` silently skips it and
/// retries after the next threshold is reached.
///
/// Unlike [`rand` v0.9's equivalent], this variant is built on top of [`TryRng`] instead of the
/// block [`Generator`], allowing a wider choice of underlying generators, including [`StdRng`].
///
/// # Examples
///
/// `ReseedingRng` is useful to replicate the reseeding behavior of [`ThreadRng`]. As of `rand`
/// v0.10.0, `ThreadRng` uses the same algorithm as [`StdRng`] and reseeds it via [`SysRng`] every
/// 64KiB of output. You can emulate this behavior by configuring `ReseedingRng` as follows:
///
/// ```rust
/// use rand::{RngExt as _, rngs::StdRng, rngs::SysRng};
/// use reseeding_rng::ReseedingRng;
///
/// let mut rng = ReseedingRng::<StdRng, _>::try_new(1024 * 64, SysRng)
///     .expect("couldn't initialize ReseedingRng due to SysRng failure");
/// println!("{:?}", rng.random::<[char; 4]>());
/// ```
///
/// # Fork safety
///
/// The underlying generator is not automatically reseeded on process fork (contrast with
/// `ReseedingRng` from `rand` v0.8 and earlier). Some applications need reseeding on fork to avoid
/// the parent and child processes generating the same sequence of random numbers. The example
/// below shows a wrapper that handles this using [the `forkguard` crate].
///
/// ```rust
/// use rand::{Rng as _, rngs::StdRng, rngs::SysRng};
///
/// struct ForkSafeReseedingRng {
///     inner: reseeding_rng::ReseedingRng<StdRng, SysRng>,
///     guard: forkguard::Guard,
/// }
///
/// impl ForkSafeReseedingRng {
///     fn next_u32(&mut self) -> u32 {
///         if self.guard.detected_fork() {
///             // reseed ReseedingRng in child process
///             let _ = self.inner.try_reseed();
///         }
///         self.inner.next_u32()
///     }
/// }
/// ```
///
/// [`rand` v0.9's equivalent]: https://docs.rs/rand/0.9.2/rand/rngs/struct.ReseedingRng.html
/// [`Generator`]: rand_core::block::Generator
/// [`StdRng`]: https://docs.rs/rand/0.10/rand/rngs/struct.StdRng.html
/// [`SysRng`]: https://docs.rs/rand/0.10/rand/rngs/struct.SysRng.html
/// [`ThreadRng`]: https://docs.rs/rand/0.10/rand/rngs/struct.ThreadRng.html
/// [the `forkguard` crate]: https://crates.io/crates/forkguard
pub struct ReseedingRng<R, Rsdr> {
    inner: R,
    reseeder: Rsdr,
    threshold: usize,
    bytes_consumed: usize,
}

impl<R, Rsdr> ReseedingRng<R, Rsdr>
where
    R: SeedableRng,
    Rsdr: TryRng,
{
    /// Creates a new instance with a reseeding threshold in bytes and a seed generator for
    /// initialization and reseeding.
    ///
    /// # Panics
    ///
    /// Panics if `threshold` is zero.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `reseeder` fails to seed the underlying generator.
    pub fn try_new(threshold: usize, mut reseeder: Rsdr) -> Result<Self, Rsdr::Error> {
        assert!(threshold > 0, "`threshold` must be greater than zero");
        R::try_from_rng(&mut reseeder).map(|inner| Self {
            inner,
            reseeder,
            threshold,
            bytes_consumed: 0,
        })
    }

    /// Reseeds the underlying generator immediately.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `reseeder` fails to seed the underlying generator.
    pub fn try_reseed(&mut self) -> Result<(), Rsdr::Error> {
        R::try_from_rng(&mut self.reseeder).map(|inner| {
            self.inner = inner;
            self.bytes_consumed = 0;
        })
    }

    #[cold]
    fn reset_after_reseed_attempt_at(&mut self, pos: usize) {
        let _ = self.try_reseed();
        self.bytes_consumed = pos;
    }
}

impl<R, Rsdr> TryRng for ReseedingRng<R, Rsdr>
where
    R: TryRng + SeedableRng,
    Rsdr: TryRng,
{
    type Error = R::Error;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        self.bytes_consumed += 32 / 8;
        if self.bytes_consumed > self.threshold {
            self.reset_after_reseed_attempt_at(32 / 8);
        }
        self.inner.try_next_u32()
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        self.bytes_consumed += 64 / 8;
        if self.bytes_consumed > self.threshold {
            self.reset_after_reseed_attempt_at(64 / 8);
        }
        self.inner.try_next_u64()
    }

    fn try_fill_bytes(&mut self, mut dst: &mut [u8]) -> Result<(), Self::Error> {
        loop {
            if self.bytes_consumed + dst.len() <= self.threshold {
                self.bytes_consumed += dst.len();
                break self.inner.try_fill_bytes(dst);
            }
            if self.bytes_consumed < self.threshold {
                let mid = self.threshold - self.bytes_consumed;
                self.bytes_consumed += mid;
                self.inner.try_fill_bytes(&mut dst[..mid])?;
                dst = &mut dst[mid..];
            }
            self.reset_after_reseed_attempt_at(0);
        }
    }
}

impl<R, Rsdr> TryCryptoRng for ReseedingRng<R, Rsdr>
where
    R: TryCryptoRng + SeedableRng,
    Rsdr: TryCryptoRng,
{
}

/// This implementation reseeds the underlying generator upon `clone()`.
impl<R, Rsdr> Clone for ReseedingRng<R, Rsdr>
where
    R: SeedableRng,
    Rsdr: Clone + Rng,
{
    fn clone(&self) -> Self {
        let mut reseeder = self.reseeder.clone();
        Self {
            inner: R::from_rng(&mut reseeder),
            reseeder,
            threshold: self.threshold,
            bytes_consumed: 0,
        }
    }
}

impl<R, Rsdr> fmt::Debug for ReseedingRng<R, Rsdr>
where
    R: fmt::Debug,
    Rsdr: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("ReseedingRng")
            .field("inner", &self.inner)
            .field("reseeder", &self.reseeder)
            .field("threshold", &self.threshold)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::{StdRng, SysRng};

    #[test]
    fn mirror_rand09_reseeding_rng() {
        use rand_chacha09::{ChaCha12Core, ChaCha12Rng};
        use rand09::{RngCore as _, SeedableRng as _};

        use mock::Rand09Adapter as Adapter;

        type OurImpl = ReseedingRng<Adapter, Adapter>;
        type TheirImpl = rand09::rngs::ReseedingRng<ChaCha12Core, ChaCha12Rng>;

        const N: usize = 1024 * 16 * 5 + 997;

        let seed = rand09::random();
        let mut o = OurImpl::try_new(1024 * 16, Adapter::from_seed(seed)).unwrap();
        let mut t = TheirImpl::new(1024 * 16, ChaCha12Rng::from_seed(seed)).unwrap();

        for _ in 0..(N / 4) {
            assert_eq!(o.next_u32(), t.next_u32());
        }

        o.try_reseed().unwrap();
        t.reseed().unwrap();

        for _ in 0..(N / 8) {
            assert_eq!(o.next_u64(), t.next_u64());
        }

        o.try_reseed().unwrap();
        t.reseed().unwrap();

        let mut buf_o = vec![0u8; 17 * 4];
        let mut buf_t = vec![0u8; buf_o.len()];
        for _ in 0..(N / buf_o.len()) {
            o.fill_bytes(&mut buf_o[..]);
            t.fill_bytes(&mut buf_t[..]);
            assert_eq!(buf_o, buf_t);
        }

        o.try_reseed().unwrap();
        t.reseed().unwrap();

        buf_o.resize(1024 * 16 * 2 + 7 * 4, 0);
        buf_t.resize(buf_o.len(), 0);
        for _ in 0..(N / buf_o.len()) {
            o.fill_bytes(&mut buf_o[..]);
            t.fill_bytes(&mut buf_t[..]);
            assert_eq!(buf_o, buf_t);
        }
    }

    #[test]
    fn reseed_after_threshold() {
        let seed = rand::random();
        let mut g1 = StdRng::from_rng(&mut StdRng::from_seed(seed));
        let mut g2 =
            ReseedingRng::<StdRng, _>::try_new(1024 * 64, StdRng::from_seed(seed)).unwrap();

        for _ in 0..(64 * 1024 / (32 / 8 + 32 / 8 + 64 / 8)) {
            assert_eq!(g1.next_u32(), g2.next_u32());
            assert_eq!(g1.next_u32(), g2.next_u32());
            assert_eq!(g1.next_u64(), g2.next_u64());
        }

        assert_ne!(g1.next_u32(), g2.next_u32());
        assert_ne!(g1.next_u64(), g2.next_u64());
    }

    #[test]
    fn count_periodic_reseeds() {
        use std::cell::Cell;

        struct MockReseeder<'a> {
            counter: &'a Cell<usize>,
        }

        impl TryRng for MockReseeder<'_> {
            type Error = <SysRng as TryRng>::Error;

            fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
                self.counter.set(self.counter.get() + 1);
                SysRng.try_next_u32()
            }

            fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
                self.counter.set(self.counter.get() + 1);
                SysRng.try_next_u64()
            }

            fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
                self.counter.set(self.counter.get() + 1);
                SysRng.try_fill_bytes(dst)
            }
        }

        let counter = Cell::new(0);
        let reseeder = MockReseeder { counter: &counter };
        let mut rng = ReseedingRng::<StdRng, _>::try_new(10, reseeder).unwrap();
        assert_eq!(counter.get(), 1);

        rng.fill_bytes(&mut [0u8; 10]);
        assert_eq!(counter.get(), 1);
        assert_eq!(rng.bytes_consumed, 10);
        rng.fill_bytes(&mut [0u8; 1]);
        assert_eq!(counter.get(), 2);
        assert_eq!(rng.bytes_consumed, 1);
        rng.fill_bytes(&mut [0u8; 9]);
        assert_eq!(counter.get(), 2);
        assert_eq!(rng.bytes_consumed, 10);
        rng.fill_bytes(&mut [0u8; 1]);
        assert_eq!(counter.get(), 3);
        assert_eq!(rng.bytes_consumed, 1);
        rng.fill_bytes(&mut [0u8; 25]);
        assert_eq!(counter.get(), 5);
        assert_eq!(rng.bytes_consumed, 6);

        rng.next_u32();
        assert_eq!(counter.get(), 5);
        assert_eq!(rng.bytes_consumed, 10);
        rng.next_u32();
        assert_eq!(counter.get(), 6);
        assert_eq!(rng.bytes_consumed, 4);
        rng.next_u32();
        assert_eq!(counter.get(), 6);
        assert_eq!(rng.bytes_consumed, 8);
        rng.next_u32(); // discarding 2 bytes
        assert_eq!(counter.get(), 7);
        assert_eq!(rng.bytes_consumed, 4);

        rng.fill_bytes(&mut [0u8; 7]);
        assert_eq!(counter.get(), 8);
        assert_eq!(rng.bytes_consumed, 1);
        rng.next_u64();
        assert_eq!(counter.get(), 8);
        assert_eq!(rng.bytes_consumed, 9);
        rng.next_u64(); // discarding 1 byte
        assert_eq!(counter.get(), 9);
        assert_eq!(rng.bytes_consumed, 8);
    }

    #[test]
    #[should_panic]
    fn panic_if_threshold_is_zero() {
        let _ = ReseedingRng::<StdRng, _>::try_new(0, SysRng);
    }

    /// Tests in this module may occasionally fail.
    mod fallible {
        use super::*;

        const N: usize = 20 * 256;

        #[test]
        fn generate_random_numbers() {
            let mut rng = ReseedingRng::<StdRng, _>::try_new(1024, SysRng).unwrap();

            let arrays = (0..N)
                .map(|_| rng.next_u32().to_le_bytes())
                .collect::<Vec<_>>();
            assert!(check_each_byte_for_randomness(&arrays));

            let arrays = (0..N)
                .map(|_| rng.next_u64().to_le_bytes())
                .collect::<Vec<_>>();
            assert!(check_each_byte_for_randomness(&arrays));

            let mut buf = [0u8; 17];
            let arrays = (0..N)
                .map(|_| {
                    rng.fill_bytes(buf.as_mut());
                    buf
                })
                .collect::<Vec<_>>();
            assert!(check_each_byte_for_randomness(&arrays));
        }

        #[test]
        fn handle_corner_cases() {
            let mut rng = ReseedingRng::<StdRng, _>::try_new(1, SysRng).unwrap();

            let arrays = (0..N)
                .map(|_| rng.next_u32().to_le_bytes())
                .collect::<Vec<_>>();
            assert!(check_each_byte_for_randomness(&arrays));

            let arrays = (0..N)
                .map(|_| rng.next_u64().to_le_bytes())
                .collect::<Vec<_>>();
            assert!(check_each_byte_for_randomness(&arrays));

            let mut buf = [0u8; 5];
            let arrays = (0..N)
                .map(|_| {
                    rng.fill_bytes(buf.as_mut());
                    buf
                })
                .collect::<Vec<_>>();
            assert!(check_each_byte_for_randomness(&arrays));

            let mut buf = [0u8; 0];
            for _ in 0..N {
                rng.fill_bytes(buf.as_mut());
            }
        }

        fn check_each_byte_for_randomness<const N: usize>(arrays: &[[u8; N]]) -> bool {
            (0..N).all(|i| {
                let mut freq = [0usize; 256];
                for array in arrays {
                    freq[array[i] as usize] += 1; // by column
                }

                let expected = arrays.len() as f64 / 256.0;
                let chi_squared = freq.iter().fold(0.0, |acc, &observed| {
                    let dev = observed as f64 - expected;
                    acc + dev * dev / expected
                });

                chi_squared < 330.52 // df = 255, p = 0.001
            })
        }
    }
}
