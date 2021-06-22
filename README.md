Helpers to read binary data files in rust

# Overview

The segsource crate is designed to make reading binary data easier. It is not
meant to replace other wonderful crates like
[bytes](https://github.com/tokio-rs/bytes),
[nom](https://github.com/Geal/nom) or [binread](https://github.com/jam1garner/binread),
but instead is meant to work with them as a single, common interface between.

This is primarily done via the [`BinReader`] trait, as well as a variety of different
implementations of it useful for a variety of purposes.

# Feature Flags

As of right now, BinReader only has two feature flags:

- `nom-support` which allows [nom](https://github.com/Geal/nom) to parse from
  BinReaders.
- `memmap` which supports platform-independent memory mapped files (via the
  [memmap2](https://github.com/RazrFalcon/memmap2-rs) crate).

**NOTE**: This is still a WIP.
