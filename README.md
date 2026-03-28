# Crate `reseeding_rng`

`ReseedingRng` that periodically reseeds the underlying pseudorandom number
generator.

This crate provides a simplified reimplementation of `ReseedingRng` for use with
the random number generators from the `rand` crate v0.10, which no longer
includes the `ReseedingRng` from v0.9 and earlier.

Note that periodic reseeding is never strictly _necessary_.
See [the `rand` v0.9 documentation] for further discussion.

[the `rand` v0.9 documentation]: https://docs.rs/rand/0.9.2/rand/rngs/struct.ReseedingRng.html

## Examples

```rust
use rand::{RngExt as _, rngs::StdRng, rngs::SysRng};
use reseeding_rng::rand010::ReseedingRng;

let mut rng = ReseedingRng::<StdRng, _>::try_new(1024 * 64, SysRng).unwrap();
println!("{:?}", rng.random::<[char; 4]>());
```

## License

Licensed under the Apache License, Version 2.0.
