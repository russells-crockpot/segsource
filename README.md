# segsource

Segsource is a crate designed to assist in reading data. Although is specifically designed with
binary (`u8`) data, it is not limited to this.

### Overview

Segsource is designed to easily and efficiently allow the reading of data without using much
memory. It does this by having two basic concepts: [`Source`]s, which own the data and
[`Segment`]s which allow access the data.

Each [`Segment`] is a thread-safe struct that retains minimal basic information such as the
position of an internal cursor (expressed via the [`Segment::current_offset`] method), a
reference to this data, etcetera.

### Feature Flags

The following features are available for segsource:

1. `async` which adds support for various `async` operations using `tokio`.
2. `derive` which includes several macros for creating structs from [`Segment`]s.
3. `mmap` which adds support for memory mapped files.
4. `std` which adds support for file and I/O operations.
5. `with_bytes` which adds support for using the `bytes` crate.

Of these, only `derive` and `std` are enabled by default.

### Why segsource?

There are various other crates out there in a similar vein to segsource, and in your use case,
some of them might be a better idea. I'll go through a few of the other options and let you
decide for yourself:

- `bytes`: `segsource` actually offers native support for `bytes` crate via the appropriately
  named `bytes` feature. While bytes is great, it does have its limitations, the two biggest
  ones being the most read operations require it to be mutable and that there's no way to go
  "back". Segsource solves both of these cases.

- `binread`: Not a replacement for `segsource` as a whole, but for the derivations provided via
  the `derive` feature. As of this writing, `binread` is more feature rich than `segsource`'s
  derives (and since [`Segment`]s extend `std::io::Seek` and `std::io::Read`, they will work
  with `binread`]. Unfortunately, this again requires the passed in

- `bitvec`: You may have noticed that you can essentially do simple memory emulation with
  `segsource (e.g. you can have an initial offset, you work in offsets, etcetera). Simple, being
  the keyword here. `bitvec` is not simple nor can it be given its scope.

- `std`: You could use various items from the standard library, such as a `Vec` or an
  `io::Cursor`, but all of these have limitations (e.g. a `Vec` can't have an initial offset and
  a can only move relative to its current position).

### Derive

Documentation is on my TODO list...

### Offsets

Instead of indexes, segsource use offsets. Depending on your use case, these will probably end
up being the same. However, you can specify an initial offset that will essentially change the
index from zero to whatever the initial_offset is.

For example:

```rust
let test_data = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
let source = SourceOfYourChoice::from_u8_slice_with_offset(&test_data, 100, Endidness::Big).
    unwrap();
let segment = source.all().unwrap();
assert_eq!(segment.u8_at(100).unwrap(), 0);
```

#### Validation

One thing you may have noticed is that we had to unwrap the value each time. This is because
methods first check to make an offset is valid. For example:

```rust
assert!(matches!(segment.u8_at(99), Err(Error::OffsetTooSmall { offset :99 })));
```

License: MIT
