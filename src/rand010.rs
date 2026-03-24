use rand_core010::{SeedableRng, TryCryptoRng, TryRng};

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
