//! [`ReseedingRng`] that periodically reseeds the underlying pseudorandom number
//! generator.
//!
//! This crate provides a simplified reimplementation of `ReseedingRng` for use with
//! the random number generators from the `rand` crate v0.10, which no longer
//! includes the `ReseedingRng` from v0.9 and earlier.
//!
//! Note that periodic reseeding is never strictly _necessary_.
//! See [the `rand` v0.9 documentation] for further discussion.
//!
//! [the `rand` v0.9 documentation]: https://docs.rs/rand/0.9.2/rand/rngs/struct.ReseedingRng.html
//!
//! # Examples
//!
//! ```rust
//! use rand::{RngExt as _, rngs::StdRng, rngs::SysRng};
//! use reseeding_rng::ReseedingRng;
//!
//! let mut rng = ReseedingRng::<StdRng, _>::try_new(1024 * 64, SysRng).unwrap();
//! println!("{:?}", rng.random::<[char; 4]>());
//! ```

#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use rand_core::{Rng, SeedableRng, TryCryptoRng, TryRng};

/// A wrapper that periodically reseeds the underlying pseudorandom number generator.
///
/// This type reseeds the underlying generator every time a specified number of random bytes have
/// been produced. If the periodic reseeding attempt fails, `ReseedingRng` silently skips it and
/// retries after the next threshold is reached.
///
/// Unlike [`rand` v0.9's equivalent](https://docs.rs/rand/0.9.2/rand/rngs/struct.ReseedingRng.html),
/// this variant is built on top of [`TryRng`] instead of the block [`Generator`], allowing a wider
/// choice of underlying generators, including [`StdRng`].
///
/// [`Generator`]: rand_core::block::Generator
/// [`StdRng`]: https://docs.rs/rand/0.10/rand/rngs/struct.StdRng.html
#[derive(Debug)]
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
    pub fn reseed(&mut self) -> Result<(), Rsdr::Error> {
        R::try_from_rng(&mut self.reseeder).map(|inner| {
            self.inner = inner;
            self.bytes_consumed = 0;
        })
    }

    #[cold]
    fn reseed_and_reset(&mut self, pos: usize) {
        let _ = self.reseed();
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
            self.reseed_and_reset(32 / 8);
        }
        self.inner.try_next_u32()
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        self.bytes_consumed += 64 / 8;
        if self.bytes_consumed > self.threshold {
            self.reseed_and_reset(64 / 8);
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
            self.reseed_and_reset(0);
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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests {
    use super::*;

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

        o.reseed().unwrap();
        t.reseed().unwrap();

        for _ in 0..(N / 8) {
            assert_eq!(o.next_u64(), t.next_u64());
        }

        o.reseed().unwrap();
        t.reseed().unwrap();

        let mut buf_o = vec![0u8; 17 * 4];
        let mut buf_t = vec![0u8; buf_o.len()];
        for _ in 0..(N / buf_o.len()) {
            o.fill_bytes(&mut buf_o[..]);
            t.fill_bytes(&mut buf_t[..]);
            assert_eq!(buf_o, buf_t);
        }

        o.reseed().unwrap();
        t.reseed().unwrap();

        buf_o.resize(1024 * 16 * 2 + 7 * 4, 0);
        buf_t.resize(buf_o.len(), 0);
        for _ in 0..(N / buf_o.len()) {
            o.fill_bytes(&mut buf_o[..]);
            t.fill_bytes(&mut buf_t[..]);
            assert_eq!(buf_o, buf_t);
        }
    }

    /// Tests in this module may occasionally fail.
    mod fallible {
        use super::*;
        use rand::rngs::{StdRng, SysRng};

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
