#![cfg(feature = "rand010")]
#![feature(test)]

extern crate test;

use test::{Bencher, black_box};

use rand_core010::{Rng as _, SeedableRng as _};

use reseeding_rng::rand010::ReseedingRng;

#[path = "../src/rand010/mock.rs"]
mod mock;

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

    #[bench]
    fn bench_next_u32_our_reseeding(b: &mut Bencher) {
        let mut g = our_reseeding();
        b.iter(|| black_box(g.next_u32()));
    }

    #[bench]
    fn bench_next_u32_rand09_reseeding(b: &mut Bencher) {
        let mut g = rand09_reseeding();
        b.iter(|| black_box(g.next_u32()));
    }

    #[bench]
    fn bench_next_u64_our_reseeding(b: &mut Bencher) {
        let mut g = our_reseeding();
        b.iter(|| black_box(g.next_u64()));
    }

    #[bench]
    fn bench_next_u64_rand09_reseeding(b: &mut Bencher) {
        let mut g = rand09_reseeding();
        b.iter(|| black_box(g.next_u64()));
    }

    #[bench]
    fn bench_fill_bytes_our_reseeding(b: &mut Bencher) {
        let mut g = our_reseeding();
        let mut buf = vec![0u8; 97 * 4];
        b.iter(|| g.fill_bytes(&mut buf[..]));
    }

    #[bench]
    fn bench_fill_bytes_rand09_reseeding(b: &mut Bencher) {
        let mut g = rand09_reseeding();
        let mut buf = vec![0u8; 97 * 4];
        b.iter(|| g.fill_bytes(&mut buf[..]));
    }
}

mod overhead {
    use super::*;

    use rand010::{rngs::StdRng, rngs::SysRng};

    fn reseeding() -> ReseedingRng<StdRng, SysRng> {
        ReseedingRng::try_new(1024 * 64, SysRng).unwrap()
    }

    fn bare_rng() -> StdRng {
        StdRng::try_from_rng(&mut SysRng).unwrap()
    }

    #[bench]
    fn bench_next_u32_reseeding(b: &mut Bencher) {
        let mut g = reseeding();
        b.iter(|| black_box(g.next_u32()));
    }

    #[bench]
    fn bench_next_u32_bare_rng(b: &mut Bencher) {
        let mut g = bare_rng();
        b.iter(|| black_box(g.next_u32()));
    }

    #[bench]
    fn bench_next_u64_reseeding(b: &mut Bencher) {
        let mut g = reseeding();
        b.iter(|| black_box(g.next_u64()));
    }

    #[bench]
    fn bench_next_u64_bare_rng(b: &mut Bencher) {
        let mut g = bare_rng();
        b.iter(|| black_box(g.next_u64()));
    }

    #[bench]
    fn bench_fill_bytes_reseeding(b: &mut Bencher) {
        let mut g = reseeding();
        let mut buf = vec![0u8; 97 * 4];
        b.iter(|| g.fill_bytes(&mut buf[..]));
    }

    #[bench]
    fn bench_fill_bytes_bare_rng(b: &mut Bencher) {
        let mut g = bare_rng();
        let mut buf = vec![0u8; 97 * 4];
        b.iter(|| g.fill_bytes(&mut buf[..]));
    }
}
