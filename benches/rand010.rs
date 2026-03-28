#![cfg(feature = "rand010")]
#![feature(test)]

extern crate test;

use rand_core010::{Rng as _, SeedableRng as _};

use reseeding_rng::rand010::ReseedingRng;

#[path = "../src/rand010/mock.rs"]
mod mock;

macro_rules! generate_benches {
    ($rng:ident, $bench_next_u32:ident, $bench_next_u64:ident, $bench_fill_bytes:ident) => {
        #[bench]
        fn $bench_next_u32(b: &mut test::Bencher) {
            let mut rng = $rng();
            b.iter(|| test::black_box(rng.next_u32()));
        }

        #[bench]
        fn $bench_next_u64(b: &mut test::Bencher) {
            let mut rng = $rng();
            b.iter(|| test::black_box(rng.next_u64()));
        }

        #[bench]
        fn $bench_fill_bytes(b: &mut test::Bencher) {
            let mut rng = $rng();
            let mut buf = vec![0u8; 97 * 4];
            b.iter(|| rng.fill_bytes(buf.as_mut()));
        }
    };
}

mod overhead {
    use super::*;

    use rand010::rngs::{StdRng, SysRng};

    fn reseeding() -> ReseedingRng<StdRng, SysRng> {
        ReseedingRng::try_new(1024 * 64, SysRng).unwrap()
    }

    fn bare_rng() -> StdRng {
        StdRng::try_from_rng(&mut SysRng).unwrap()
    }

    generate_benches!(
        reseeding,
        bench_next_u32_reseeding,
        bench_next_u64_reseeding,
        bench_fill_bytes_reseeding
    );

    generate_benches!(
        bare_rng,
        bench_next_u32_bare_rng,
        bench_next_u64_bare_rng,
        bench_fill_bytes_bare_rng
    );
}

mod vs_rand09 {
    use super::*;

    use rand_chacha09::{ChaCha12Core, ChaCha12Rng};
    use rand09::{RngCore as _, SeedableRng as _};

    use mock::Rand09Adapter;

    fn our_reseeding() -> ReseedingRng<Rand09Adapter, Rand09Adapter> {
        let reseeder = Rand09Adapter::from_seed(rand09::random());
        ReseedingRng::try_new(1024 * 64, reseeder).unwrap()
    }

    fn rand09_reseeding() -> rand09::rngs::ReseedingRng<ChaCha12Core, ChaCha12Rng> {
        let reseeder = ChaCha12Rng::from_seed(rand09::random());
        rand09::rngs::ReseedingRng::new(1024 * 64, reseeder).unwrap()
    }

    generate_benches!(
        our_reseeding,
        bench_next_u32_our_reseeding,
        bench_next_u64_our_reseeding,
        bench_fill_bytes_our_reseeding
    );

    generate_benches!(
        rand09_reseeding,
        bench_next_u32_rand09_reseeding,
        bench_next_u64_rand09_reseeding,
        bench_fill_bytes_rand09_reseeding
    );
}
