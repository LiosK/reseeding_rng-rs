#![cfg(feature = "rand010")]
#![feature(test)]

extern crate test;

#[path = "../src/rand010/test_adapter.rs"]
mod test_adapter;

use test::{Bencher, black_box};

use rand_chacha09::{ChaCha12Core, ChaCha12Rng};
use rand_core010::{Rng as _, SeedableRng as _};
use rand09::{RngCore as _, SeedableRng as _};

use reseeding_rng::rand010::ReseedingRng;
use test_adapter::Adapter;

fn our_reseeding() -> ReseedingRng<Adapter, Adapter> {
    let reseeder = Adapter::from_seed(rand09::random());
    ReseedingRng::try_new(1024 * 64, reseeder).unwrap()
}

fn rand09_reseeding() -> rand09::rngs::ReseedingRng<ChaCha12Core, ChaCha12Rng> {
    let reseeder = ChaCha12Rng::from_seed(rand09::random());
    rand09::rngs::ReseedingRng::new(1024 * 64, reseeder).unwrap()
}

fn bare_rng() -> ChaCha12Rng {
    ChaCha12Rng::from_seed(rand09::random())
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
fn bench_next_u32_bare_rng(b: &mut Bencher) {
    let mut g = bare_rng();
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
fn bench_next_u64_bare_rng(b: &mut Bencher) {
    let mut g = bare_rng();
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

#[bench]
fn bench_fill_bytes_bare_rng(b: &mut Bencher) {
    let mut g = bare_rng();
    let mut buf = vec![0u8; 97 * 4];
    b.iter(|| g.fill_bytes(&mut buf[..]));
}
