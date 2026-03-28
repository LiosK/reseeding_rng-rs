use rand_chacha09::ChaCha12Rng;
use rand09::{RngCore as _, SeedableRng as _};

#[derive(Debug)]
pub struct Rand09Adapter(ChaCha12Rng);

impl rand_core::SeedableRng for Rand09Adapter {
    type Seed = <ChaCha12Rng as rand09::SeedableRng>::Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        Self(ChaCha12Rng::from_seed(seed))
    }
}

impl rand_core::TryRng for Rand09Adapter {
    type Error = rand_core::Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.0.next_u32())
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Ok(self.0.next_u64())
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        self.0.fill_bytes(dst);
        Ok(())
    }
}
