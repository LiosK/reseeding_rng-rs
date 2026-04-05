# Crate `reseeding_rng`

[![Crates.io](https://img.shields.io/crates/v/reseeding_rng)](https://crates.io/crates/reseeding_rng)
[![License](https://img.shields.io/crates/l/reseeding_rng)](https://github.com/LiosK/reseeding_rng-rs/blob/main/LICENSE)

`ReseedingRng` that periodically reseeds the underlying pseudorandom number
generator.

```rust
use rand::{RngExt as _, rngs::StdRng, rngs::SysRng};
use reseeding_rng::ReseedingRng;

let mut rng = ReseedingRng::<StdRng, _>::try_new(1024 * 64, SysRng)
    .expect("couldn't initialize ReseedingRng due to SysRng failure");
println!("{:?}", rng.random::<[char; 4]>());
```

This crate provides a simplified reimplementation of `ReseedingRng` for use with
the random number generators from the `rand` crate v0.10, which no longer
includes [the `ReseedingRng` from v0.9] and earlier.

This crate is `no_std`-compatible.

[the `ReseedingRng` from v0.9]: https://docs.rs/rand/0.9.2/rand/rngs/struct.ReseedingRng.html

## License

Licensed under the Apache License, Version 2.0.
