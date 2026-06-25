# Changelog

## v0.10.7 - 2026-06-24

- Minor refactoring and performance improvement.

## v0.10.6 - 2026-05-07

- Added `#[track_caller]` to improve panic diagnosis.

## v0.10.5 - 2026-05-05

- Added `std_rng` cargo feature that enables:
  - `StdReseedingRng`, a newtype with sensible defaults for general use.
  - Re-export of `rand::RngExt` trait for use with `StdReseedingRng`.
- Updated dependencies.

## v0.10.4 - 2026-04-05

- Documentation improvement.

## v0.10.3 - 2026-03-30

- Hid `bytes_consumed` from the `Debug` output.
- Minor refactoring, test improvements, and documentation updates.

## v0.10.2 - 2026-03-29

- Initial fully functional release for use with `rand` v0.10.
