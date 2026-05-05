use core::{convert, fmt};

use rand::rngs::{StdRng, SysError, SysRng};

use super::{ReseedingRng, TryCryptoRng, TryRng};

/// A newtype wrapping `ReseedingRng<StdRng, SysRng>` with a default reseeding threshold of 64KiB.
///
/// This type configures [`ReseedingRng`] with sensible defaults for general use and provides
/// specialized constructors that treat [`SysRng`] failure (highly unlikely in practice) as a panic
/// rather than an `Err`.
///
/// Note that the inner generators and the reseeding threshold may change in the future and such a
/// change may not be considered a breaking change.
///
/// # Examples
///
/// ```rust
/// use reseeding_rng::{RngExt as _, StdReseedingRng};
///
/// let mut rng = StdReseedingRng::new();
/// println!("{:?}", rng.random::<[char; 4]>());
/// ```
pub struct StdReseedingRng {
    inner: ReseedingRng<StdRng, SysRng>,
}

impl StdReseedingRng {
    /// Creates a new instance.
    ///
    /// # Panics
    ///
    /// Panics if [`SysRng`] fails to seed the underlying generator.
    pub fn new() -> Self {
        Self::try_new().expect("couldn't initialize StdReseedingRng due to SysRng failure")
    }

    /// Creates a new instance.
    ///
    /// # Errors
    ///
    /// Returns `Err` if [`SysRng`] fails to seed the underlying generator.
    pub fn try_new() -> Result<Self, SysError> {
        ReseedingRng::try_new(1024 * 64, SysRng).map(|inner| Self { inner })
    }

    /// Reseeds the underlying generator immediately.
    ///
    /// # Errors
    ///
    /// Returns `Err` if [`SysRng`] fails to seed the underlying generator.
    pub fn try_reseed(&mut self) -> Result<(), SysError> {
        self.inner.try_reseed()
    }
}

impl TryRng for StdReseedingRng {
    type Error = convert::Infallible;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        self.inner.try_next_u32()
    }

    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        self.inner.try_next_u64()
    }

    #[inline]
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        self.inner.try_fill_bytes(dst)
    }
}

impl TryCryptoRng for StdReseedingRng {}

/// This implementation reseeds the underlying generator upon `clone()` and panics if [`SysRng`]
/// fails to do so.
impl Clone for StdReseedingRng {
    fn clone(&self) -> Self {
        Self::new()
    }
}

/// This implementation panics if [`SysRng`] fails to seed the underlying generator.
impl Default for StdReseedingRng {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for StdReseedingRng {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("StdReseedingRng").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_hides_inner_state() {
        let s = format!("{:?}", StdReseedingRng::new());
        assert_eq!(s, "StdReseedingRng { .. }");
    }

    /// Tests in this module may occasionally fail.
    mod fallible {
        use super::*;
        use crate::tests::check_each_byte_for_randomness;
        use rand_core::Rng as _;

        const N: usize = 40 * 1024;

        #[test]
        fn generate_random_numbers() {
            let mut rng = StdReseedingRng::new();

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
    }
}
