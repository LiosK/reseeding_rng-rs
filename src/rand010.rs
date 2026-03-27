use rand_core010::{Rng, SeedableRng, TryCryptoRng, TryRng};

/// ```rust
/// # use rand010 as rand;
/// use rand::{RngExt as _, rngs::StdRng, rngs::SysRng};
/// use reseeding_rng::rand010::ReseedingRng;
///
/// let mut rng = ReseedingRng::<StdRng, _>::try_new(1024 * 64, SysRng).unwrap();
/// println!("{:?}", rng.random::<[char; 4]>());
/// ```
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
    pub fn try_new(threshold: usize, mut reseeder: Rsdr) -> Result<Self, Rsdr::Error> {
        assert!(threshold > 0, "`threshold` must be greater than zero");
        R::try_from_rng(&mut reseeder).map(|inner| Self {
            inner,
            reseeder,
            threshold,
            bytes_consumed: 0,
        })
    }

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
            if self.bytes_consumed >= self.threshold {
                self.reseed_and_reset(0);
            }
            let len = dst.len().min(self.threshold - self.bytes_consumed);
            self.bytes_consumed += len;
            self.inner.try_fill_bytes(&mut dst[..len])?;
            dst = &mut dst[len..];
            if dst.is_empty() {
                break Ok(());
            }
        }
    }
}

impl<R, Rsdr> TryCryptoRng for ReseedingRng<R, Rsdr>
where
    R: TryCryptoRng + SeedableRng,
    Rsdr: TryCryptoRng,
{
}

/// This implementation reseeds the inner generator upon `clone()`.
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
        use rand010::rngs::{StdRng, SysRng};

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
