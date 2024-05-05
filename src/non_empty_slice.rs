use std::num::NonZeroUsize;

use crate::non_empty_vec::*;

/// A continuous non-empty slice.
/// 
/// This type is a thin wrapper directly around `[T]`, and as such is un`Sized`. 
/// To use it, it'll have to be behind some form of indirection, 
/// such as `&NonEmptySlice`, `&mut NonEmptySlice` or `Box<NonEmptySlice>`.
/// 
/// Indexing with a range is only possible using `RangeFull` (`[..]`) and `RangeToInclusive` (`[..=y]`), 
/// always returning a `NonEmptySlice`. No other range is supported, since they have the possibility of
/// being empty, and returning different types depending on the range would be confusing behaviour.
/// To get a regular, possibly empty slice from indexing, consider doing `&self.get_slice()[x..y]`.
#[repr(transparent)]
#[derive(Hash)]
pub struct NonEmptySlice<T>(pub(crate) [T]);

impl<T> NonEmptySlice<T> {
    /// Using this in a const context is not reccomended, just 
    /// use `new` instead and handle the error accordingly.
    /// # Safety
    /// The slice must not be empty.
    #[inline]
    pub const unsafe fn from_slice_unchecked(slice: &[T]) -> &NonEmptySlice<T> {
        unsafe { std::mem::transmute(slice) }
    }

    /// # Safety
    /// The slice must not be empty
    #[inline]
    pub unsafe fn from_slice_unchecked_mut(slice: &mut [T]) -> &mut NonEmptySlice<T> {
        unsafe { std::mem::transmute(slice) }
    }

    /// Creates a new `&NonEmptySlice`, from a slice, returning `None` if the slice is empty.
    #[inline]
    pub const fn try_from_slice(slice: &[T]) -> Result<&NonEmptySlice<T>, &[T]> {
        if slice.is_empty() {
            Err(slice)
        } else {
            Ok(unsafe { NonEmptySlice::from_slice_unchecked(slice) })
        }
    }

    /// Creates a new `&mut NonEmptySlice`, from a slice, returning `None` if the slice is empty.
    #[inline]
    pub fn try_from_slice_mut(slice: &mut [T]) -> Result<&mut NonEmptySlice<T>, &mut [T]> {
        if slice.is_empty() {
            Err(slice)
        } else {
            Ok(unsafe { NonEmptySlice::from_slice_unchecked_mut(slice) })
        }
    }

    /// Gets the underlying slice reference behind the `NonEmptySlice`.
    /// This type implements `Deref<Target = [T]>`, consider simply borrowing the value.
    #[inline]
    pub const fn get_slice(&self) -> &[T] {
        &self.0
    }

    /// Gets the underlying mutable slice reference behind the `NonEmptySlice`.
    /// This type implements `DerefMut<Target = [T]>`, consider simply borrowing the value.
    #[inline]
    pub fn get_slice_mut(&mut self) -> &mut [T] {
        &mut self.0
    }

    /// Returns the number of elements in the slice, guaranteeing that it won't be 0.
    /// If you need the result to be a `usize`, use `get_len` instead.
    #[inline]
    pub const fn len(&self) -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(self.0.len()) }
    }

    /// Returns the number of elements in the slice, as a `usize`.
    /// If you need the result to be a `NonZeroUsize`, use `len` instead.
    #[inline]
    pub const fn get_len(&self) -> usize {
        self.len().get()
    }

    /// Returns true if the slice has just 1 element. \
    /// Used internaly to check if removing an item would cause the object to become empty.
    /// 
    /// # Examples
    ///
    /// ```
    /// let one_element = NonEmptySlice::new(&[1]).unwrap();
    /// let three_elements = NonEmptySlice::new(&[1, 2, 3]).unwrap();
    /// let two_elements = NonEmptySlice::new(&[1, 2]).unwrap();
    /// 
    /// assert!(one_element.has_just_1_element());
    /// assert_ne!(three_elements.has_just_1_element());
    /// assert_ne!(two_elements.has_just_1_element());
    /// ```
    #[inline]
    pub const fn has_just_1_element(&self) -> bool {
        self.get_len() == 1
    }

    // pub const IS_EMPTY: bool = false;

    /// Returns a reference to the first element in the slice.
    /// The slice is guaranteed to have at least 1 item, so this method is infallible.
    #[inline]
    pub fn first(&self) -> &T {
        unsafe { self.get_unchecked(0) }
    }

    /// Returns a mutable reference to the first element in the slice.
    /// The slice is guaranteed to have at least 1 item, so this method is infallible.
    #[inline]
    pub fn first_mut(&mut self) -> &mut T {
        unsafe { self.get_unchecked_mut(0) }
    }

    /// Returns a reference to the last element in the slice.
    /// The slice is guaranteed to have at least 1 item, so this method is infallible.
    #[inline]
    pub fn last(&self) -> &T {
        unsafe { self.get_unchecked(self.get_len() - 1) }
    }

    /// Returns a mutable reference to the last element in the slice.
    /// The slice is guaranteed to have at least 1 item, so this method is infallible.
    #[inline]
    pub fn last_mut(&mut self) -> &mut T {
        let last_index = self.get_len() - 1;
        unsafe { self.get_unchecked_mut(last_index) }
    }

    /// `clone`s all elements of the slice into a new vector, 
    /// guaranteeing that the resulting vector is not empty.
    #[inline]
    pub fn to_vec(&self) -> NonEmptyVec<T> where T: Clone {
        NonEmptyVec(self.0.to_vec())
    }

    /// Safely converts a `Box<NonEmptySlice>` into a `NonEmptyVec`, upholding non-emptyness guarantees.
    #[inline]
    pub fn into_vec(self: Box<Self>) -> NonEmptyVec<T> {
        let mut self_box = std::mem::ManuallyDrop::new(self);
        let vec = unsafe { Vec::<T>::from_raw_parts(self_box.0.as_mut_ptr(), self_box.0.len(), self_box.0.len()) };
        NonEmptyVec(vec)
    }

    /// `clone`s all elements of the slice into a new vector, repeated `n` times.
    /// The resulting vector is guaranteed not to be empty.
    #[inline]
    pub fn repeat(&self, n: NonZeroUsize) -> NonEmptyVec<T> where T: Copy {
        NonEmptyVec(self.0.repeat(n.get()))
    }
}

impl<T> std::ops::Deref for NonEmptySlice<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for NonEmptySlice<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for NonEmptySlice<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.0)
    }
}



impl<T> std::ops::Index<usize> for NonEmptySlice<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> std::ops::IndexMut<usize> for NonEmptySlice<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}



/// Returns self and as such is guaranteed to have at least 1 item.
impl<T> std::ops::Index<std::ops::RangeFull> for NonEmptySlice<T> {
    type Output = NonEmptySlice<T>;

    #[inline]
    fn index(&self, _index: std::ops::RangeFull) -> &Self::Output {
        self
    }
}

impl<T> std::ops::IndexMut<std::ops::RangeFull> for NonEmptySlice<T> {
    #[inline]
    fn index_mut(&mut self, _index: std::ops::RangeFull) -> &mut Self::Output {
        self
    }
}



// RangeToInclusive is guaranteed to have at least 1 item.
impl<T> std::ops::Index<std::ops::RangeToInclusive<usize>> for NonEmptySlice<T> {
    type Output = NonEmptySlice<T>;

    #[inline]
    fn index(&self, index: std::ops::RangeToInclusive<usize>) -> &Self::Output {
        unsafe { NonEmptySlice::from_slice_unchecked(&self.0[index]) }
    }
}

impl<T> std::ops::IndexMut<std::ops::RangeToInclusive<usize>> for NonEmptySlice<T> {
    #[inline]
    fn index_mut(&mut self, index: std::ops::RangeToInclusive<usize>) -> &mut Self::Output {
        unsafe { NonEmptySlice::from_slice_unchecked_mut(&mut self.0[index]) }
    }
}



impl<'a, T> TryFrom<&'a [T]> for &'a NonEmptySlice<T> {
    type Error = &'a [T];

    fn try_from(slice: &'a [T]) -> Result<Self, Self::Error> {
        NonEmptySlice::try_from_slice(slice)
    }
}

impl<'a, T> TryFrom<&'a mut [T]> for &'a NonEmptySlice<T> {
    type Error = &'a mut [T];

    fn try_from(slice: &'a mut [T]) -> Result<Self, Self::Error> {
        NonEmptySlice::try_from_slice_mut(slice).map(|x| &*x)
    }
}

impl<'a, T> TryFrom<&'a mut [T]> for &'a mut NonEmptySlice<T> {
    type Error = &'a mut [T];

    fn try_from(slice: &'a mut [T]) -> Result<Self, Self::Error> {
        NonEmptySlice::try_from_slice_mut(slice)
    }
}

impl<'a, T> From<&'a NonEmptySlice<T>> for &'a [T] {
    fn from(non_empty_slice: &'a NonEmptySlice<T>) -> &'a [T] {
        non_empty_slice
    }
}

impl<'a, T> From<&'a mut NonEmptySlice<T>> for &'a [T] {
    fn from(non_empty_slice: &'a mut NonEmptySlice<T>) -> &'a [T] {
        non_empty_slice
    }
}

impl<'a, T> From<&'a mut NonEmptySlice<T>> for &'a mut [T] {
    fn from(non_empty_slice: &'a mut NonEmptySlice<T>) -> &'a mut [T] {
        non_empty_slice
    }
}



#[cfg(feature = "static_assert_generic")]
use static_assert_generic::static_assert;

#[cfg(feature = "static_assert_generic")]
impl<T> NonEmptySlice<T> {
    /// Creates a NonEmptySlice from an array whose length is not 0.\
    /// The length of the array is checked at compile time, and as such this method is infalible.
    /// If the length of the array is not 0, a compiler error will be given. This requires a full build and does not show up when running `cargo check`.
    #[inline]
    pub fn from_arr<const N: usize>(arr: &[T; N]) -> &NonEmptySlice<T> {
        static_assert!((N: usize) N != 0 => "Length of array must be non-zero to create NonEmptySlice.");
        unsafe { NonEmptySlice::from_slice_unchecked(arr) }
    }
}

#[cfg(feature = "static_assert_generic")]
impl<'a, T: Clone, const N: usize> From<&'a [T; N]> for &'a NonEmptySlice<T> {
    #[inline]
    fn from(value: &'a [T; N]) -> Self {
        NonEmptySlice::from_arr(value)
    }
}