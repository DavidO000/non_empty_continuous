# non_empty_continuous

Crate for non-empty continuous collections.

Being continuous ends up being useful as this way converting between a type and its regular counterpart becomes a zero-cost operation.
All types utilise `#[repr(transparent)]`.

This crate attempts to reimplement as much functionality as possible from the non-empty counterparts. In many cases, they are drop-in replacements.

## Examples

```rust
let first_element = 10;
let mut non_empty_vec: NonEmptyVec<i32> = NonEmptyVec::new(first_element);
non_empty_vec.reserve(2);
non_empty_vec.push(20);
non_empty_vec.push(30);
_ = non_empty_vec.try_pop();

let non_empty_slice: &NonEmptySlice<i32> = &non_empty_vec[..=1];
let non_empty_slice_mut: &mut NonEmptySlice<i32> = &mut non_empty_vec[..];

let length: std::num::NonZeroUsize = non_empty_slice.len();

let non_empty_vec_from_macro = ne_vec![99, 98, 97];
```

## Features

### `smallvec`
Exposes `NonEmptySmallVec`, a non-empty wrapper around `SmallVec` from the `small_vec` crate.

```rust
let first_element = 10;
let mut non_empty_small_vec: NonEmptySmallVec<[usize; 5]> = NonEmptySmallVec::new(first_element);
non_empty_small_vec.reserve(2);
non_empty_small_vec.push(20);
non_empty_small_vec.push(30);

let _: bool = non_empty_small_vec.spilled();

let non_empty_slice: &NonEmptySlice<i32> = &non_empty_small_vec[..=1];
let non_empty_slice_mut: &mut NonEmptySlice<i32> = &mut non_empty_small_vec[..];

let non_empty_smallvec_from_macro = ne_smallvec![99, 98, 97];
```

### `static_assert_generic`
Allows for infalible operations with arrays whose length is checked not to be 0 at compile-time.

```rust
let arr = [1, 2, 3];
let mut non_empty_vec: NonEmptyVec<i32> = NonEmptyVec::from_arr(arr);
```

\
Empty arrays give an error.
```rust
let arr2 = [];
let mut non_empty_vec: NonEmptyVec<i32> = NonEmptyVec::from_arr(arr); // !!!
```

\
Smallvec `static_assert_generic` featues also requires `smallvec`'s `const_generics` feature.
```rust
let arr3 = [4, 5, 6];
let mut non_empty_small_vec: NonEmptySmallVec<i32> = NonEmptySmallVec::from_arr(arr3);
```

License: 0BSD
