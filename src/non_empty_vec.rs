use std::num::NonZeroUsize;

use crate::non_empty_slice::*;

/// The easiest way to create a non-empty vec.
/// An error will be raised if no elements are porvided.
/// Repeating syntax requires a `NonZeroUsize`.
/// 
/// # Examples
/// ```
/// let non_empty_vec_from_macro = ne_vec![99, 98, 97];
/// let non_empty_vec_from_macro2 = ne_vec![0; std::num::NonZeroUsize::new(100).unwrap()];
/// let _ = ne_vec![]; // Error: Cannot make an empty NonEmptyVec
/// ```
#[macro_export]
macro_rules! ne_vec {
    ($($item: expr),+ $(,)?) => {
        unsafe { $crate::NonEmptySmallVec::from_array_unchecked([$($item),+]) }
    };
    ($item: expr; $amount: expr) => {
        $crate::NonEmptyVec::from_elem($item, $amount)
    };
    () => {
        compile_error!("Cannot make an empty NonEmptyVec");
    }
}

/// A continuous non-empty vector.
/// 
/// Getting direct mutable acces to the inner vector is not allowed, 
/// since that way setting the size of the vector to 0 becomes possible.
/// As such, many methods that mutate the inner vector are re-implemented.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NonEmptyVec<T>(pub(crate) Vec<T>);

impl<T> NonEmptyVec<T> {
    /// Creates a new NonEmptyVec, with precisely one element inside of it. 
    /// If you're starting off with more than one item, consider using 
    /// `NonEmptyVec::with_capacity`, `NonEmptyVec::from(array)`, or the `ne_vec!` macro.
    #[inline]
    pub fn new(item: T) -> NonEmptyVec<T> {
        NonEmptyVec(vec![item])
    }

    /// Creates a new NonEmptyVec, with precisely one element inside of it, and a 
    /// stated capacity (unless `capacity` is 0, in which case the actual capacity will be 1).
    /// If `capacity` is 1, this is the same as calling `NonEmptyVec::new`.
    /// If you need to specify the exact capacity in all cases, use `with_exact_capacity` instead.
    #[inline]
    pub fn with_capacity(item: T, capacity: usize) -> NonEmptyVec<T> {
        let mut vec = Vec::with_capacity(capacity);
        vec.push(item);
        NonEmptyVec(vec)
    }

    /// Creates a new NonEmptyVec, with precisely one element inside of it, and a stated non-zero capacity.
    /// If `capacity` is 1, this is the same as calling `NonEmptyVec::new`.
    /// If there isn't an issue with allocating 1 element when your 
    /// `capacity` variable chould be 0, use `with_capacity` instead.
    #[inline]
    pub fn with_exact_capacity(item: T, capacity: NonZeroUsize) -> NonEmptyVec<T> {
        NonEmptyVec::with_capacity(item, capacity.get())
    }

    /// Safely turns a `Vec` into a `NonEmptyVec` if the vector is not empty, 
    /// otherwise an `Err` containing the original vector is returned.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let non_empty_vec: Vec<i32> = vec![1, 2, 3];
    /// assert_eq!(NonEmptyVec::try_from_vec(non_empty_vec), Ok(NonEmptyVec::from_arr([1, 2, 3]));
    /// 
    /// let empty_vec: Vec<i32> = vec![];
    /// assert_eq!(NonEmptyVec::try_from_vec(empty_vec), Err(vec![]));
    /// ```
    #[inline]
    pub fn try_from_vec(vec: Vec<T>) -> Result<NonEmptyVec<T>, Vec<T>> {
        if vec.is_empty() { Err(vec) }
        else { Ok(NonEmptyVec(vec)) }
    }

    /// If `vec` is empty, the vector is not modified and `None` is returned. 
    /// Otherwise, `vec`'s items are moved to the new `NonEmptyVec` and `vec` is emptied.
    #[inline]
    pub fn try_from_vec_ref_mut(vec: &mut Vec<T>) -> Option<NonEmptyVec<T>> {
        if vec.is_empty() { None }
        else {
            let mut non_empty_vec = NonEmptyVec(Vec::new());
            std::mem::swap(vec, &mut non_empty_vec.0);
            Some(non_empty_vec)
        }
    }

    /// # Safety
    /// `vec` must not be empty.
    #[inline]
    pub unsafe fn from_vec_unchecked(vec: Vec<T>) -> NonEmptyVec<T> {
        NonEmptyVec(vec)
    }

    /// For use if the `static_assert_generic` feature is not used. It is highly encouraged that `from_arr` is used instead.
    /// # Safety
    /// The length of the array must not be 0.
    #[inline]
    pub unsafe fn from_array_unchecked<const N: usize>(arr: [T; N]) -> NonEmptyVec<T> {
        NonEmptyVec(Vec::from(arr))
    }

    // `get_vec_mut` cannot be implemented, since it would 
    // allow for making the vec empty without unsafe.

    /// Creates a `NonEmptyVec` from raw parts, ensuring that its length and capacity are above 0.
    /// # Safety
    /// This comes with the same requirements as `Vec::from_raw_parts`
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut T, length: NonZeroUsize, capacity: NonZeroUsize) -> NonEmptyVec<T> {
        NonEmptyVec(Vec::from_raw_parts(ptr, length.get(), capacity.get()))
    }

    /// Moves the inner vector out of the `NonEmptyVec`.
    #[inline]
    pub fn to_vec(self) -> Vec<T> {
        self.0
    }

    /// Gives a read-only reference to the inner vector.
    #[inline]
    pub const fn get_vec(&self) -> &Vec<T> {
        &self.0
    }

    // Getting a mutable reference to the inner vec is not 
    // allowed since it may be modified to become empty

    /// Returns the capacity of the vector, which is guaranteed not to be 0.
    #[inline]
    pub fn capacity(&self) -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(self.0.capacity()) }
    }

    // `additional` does not need to be non-zero (goes for all subcequent methods)

    /// Wrapper around `Vec::reserve`, reimplemented since a direct mutable reference cannot be given to the underlying vector.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    /// Wrapper around `Vec::reserve_exact`, reimplemented since a direct mutable reference cannot be given to the underlying vector.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional)
    }

    /// Wrapper around `Vec::try_reserve`, reimplemented since a direct mutable reference cannot be given to the underlying vector.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize)  -> Result<(), std::collections::TryReserveError> {
        self.0.try_reserve(additional)
    }

    /// Wrapper around `Vec::try_reserve_exact`, reimplemented since a direct mutable reference cannot be given to the underlying vector.
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize)  -> Result<(), std::collections::TryReserveError> {
        self.0.try_reserve_exact(additional)
    }

    /// Wrapper around `Vec::shrink_to_fit`, reimplemented since a direct mutable reference cannot be given to the underlying vector.
    /// This only affects the vector's capacity, and as such is safe to use.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// Wrapper around `Vec::shrink_to`, reimplemented since a direct mutable reference cannot be given to the underlying vector.
    /// This only affects the vector's capacity, and as such is safe to use.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity)
    }

    /// Converts the vector into `Box<NonEmptySlice<T>>`, preserving non-emptyness guarantees.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<NonEmptySlice<T>> {
        unsafe { 
            std::mem::transmute::<
                Box<[T]>, 
                Box<NonEmptySlice<T>>
            >(self.0.into_boxed_slice()) 
        }
    }

    /// Wrapper around `Vec::truncate`. If `len` were 0, that would cause the vector to become empty.
    #[inline]
    pub fn truncate(&mut self, len: NonZeroUsize) {
        self.0.truncate(len.get())
    }

    /// Gets the underlying slice pointed to by the vector.
    /// This type implements `Deref<Target = NonEmptySlice<T>`, consider simply borrowing the value.
    #[inline]
    pub fn as_slice(&self) -> &NonEmptySlice<T> {
        self
    }

    /// Gets the underlying slice pointed to by the vector.
    /// This type implements `DerefMut<Target = NonEmptySlice<T>`, consider simply borrowing the value.
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut NonEmptySlice<T> {
        self
    }

    /// Wrapper around `Vec::set_len`.
    /// # Safety
    /// This comes with the same requirements as `Vec::set_len`.
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: NonZeroUsize) {
        self.0.set_len(new_len.get())
    }

    // `Vec::retain` cannot be implemented since the function may retain no items.

    /// Wrapper around `Vec::dedup_by_key`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    /// Dedup cannot leave the vector empty so this method is safe to use.
    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, key: F) where F: FnMut(&mut T) -> K, K: PartialEq {
        self.0.dedup_by_key(key)
    }

    /// Wrapper around `Vec::dedup_by`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    /// Dedup cannot leave the vector empty so this method is safe to use.
    #[inline]
    pub fn dedup_by<F>(&mut self, same_bucket: F) where F: FnMut(&mut T, &mut T) -> bool {
        self.0.dedup_by(same_bucket)
    }

    /// Wrapper around `Vec::push`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    /// Pushes an element to the end of the vector, reallocatig if needed.
    #[inline]
    pub fn push(&mut self, value: T) {
        self.0.push(value)
    }

    /// Wrapper around `Vec::insert`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    /// Inserts the element at the given index, shifting items as needed.
    #[inline]
    pub fn insert(&mut self, index: usize, element: T) {
        self.0.insert(index, element)
    }

    /// Wrapper around `Vec::append`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    /// This method empties `other`, meaning it cannot be a `NonEmptyVec`.
    #[inline]
    pub fn append_vec(&mut self, other: &mut Vec<T>) {
        self.0.append(other)
    }

    /// Safe wrapper around `Vec::pop`.\
    /// Returns `None` and does not pop the element if this would cause the vector to become empty.
    #[inline]
    pub fn try_pop(&mut self) -> Option<T> {
        if self.has_just_1_element() {
            None
        } else {
            self.0.pop()
        }
    }

    /// Wrapper around `Vec::swap_remove`.\
    /// This method ensures that it won't cause the vector to become empty by not allowing the first element to be removed.
    /// For a mehod that accepts any index, use `NonEmptyVec::try_swap_remove`.
    #[inline]
    pub fn swap_remove(&mut self, index: NonZeroUsize) -> T {
        self.0.swap_remove(index.get())
    }

    /// Safe wrapper around `Vec::swap_remove`.\
    /// Returns `None` if this would cause the vector to become empty.
    /// Otherwise, moves out the element at `index` and replaces it with the last element in the vector.
    #[inline]
    pub fn try_swap_remove(&mut self, index: usize) -> Option<T> {
        if self.has_just_1_element() {
            None
        } else {
            Some(self.0.swap_remove(index))
        }
    }

    /// Unsafe wrapper around `Vec::swap_remove`.\
    /// For a safe version of this method, use `NonEmptyVec::try_swap_remove`.
    /// # Safety
    /// Running this must not cause the vector to become empty.
    #[inline]
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
        self.0.swap_remove(index)
    }

    /// Safe wrapper around `Vec::remove`.\
    /// Returns `None` if this would cause the vector to become empty.
    #[inline]
    pub fn try_remove(&mut self, index: usize) -> Option<T> {
        if self.has_just_1_element(){
            None
        } else {
            Some(self.0.remove(index))
        }
    }

    /// Unsafe wrapper around `Vec::remove`.\
    /// For a safe version of this method, use `NonEmptyVec::try_remove`.
    /// # Safety
    /// Running this must not cause the vector to become empty.
    #[inline]
    pub unsafe fn remove_unchecked(&mut self, index: usize) -> T {
        self.0.remove(index)
    }

    /// Safe wrapper around  `Vec::drain`.\
    /// This method returns `None` and does not remove any elements if it coveres the whole vector.
    #[inline]
    pub fn drain<R: std::ops::RangeBounds<usize>>(&mut self, range: R) -> Option<std::vec::Drain<'_, T>> {
        if range.contains(&0) && range.contains(&(self.len().get() - 1)) {
            None
        } else {
            Some(self.0.drain(range))
        }
    }

    /// Unsafe wrapper around `Vec::drain.`\
    /// For a safe version of this method, use `NonEmptyVec::drain`.
    /// # Safety
    /// `range` must not take up the entire vector.
    #[inline]
    pub unsafe fn drain_unchecked<R: std::ops::RangeBounds<usize>>(&mut self, range: R) -> std::vec::Drain<'_, T> {
        self.0.drain(range)
    }

    // `Vec::clear` cannot be implemented for obvious reasons.

    // `Vec::len` won't be implemented since `NonEmptySlice` already 
    // implements it, and `Self` implements `Deref<Target = NonEmptySlice>`

    // const IS_EMPTY: bool = false;

    /// If `at` was 0 all items of `self` would be moved into the new vec, leaving `self` empty.
    /// This cannot return NonZeroVec since that would require at to be at least 2,
    /// and at that point you might as well do `NonZeroUsize::try_new(self.split_off(at))`
    #[inline]
    pub fn split_off(&mut self, at: NonZeroUsize) -> Vec<T> {
        self.0.split_off(at.get())
    }

    /// Safe wrapper around `Vec::resize_with`, that ensures the vector cannot become empty.
    #[inline]
    pub fn resize_with<F: FnMut() -> T>(&mut self, new_len: NonZeroUsize, f: F) {
        self.0.resize_with(new_len.get(), f)
    }

    /// Wrapper around `Vec::leak`, that preserves non-emptyness guarantees.
    #[inline]
    pub fn leak<'a>(self) -> &'a mut NonEmptySlice<T> {
        unsafe { NonEmptySlice::from_slice_unchecked_mut(self.0.leak()) }
    }

    /// Wrapper around `Vec::new_unchecked_mut`, that preserves non-emptyness guarantees.
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut NonEmptySlice<std::mem::MaybeUninit<T>> {
        unsafe { NonEmptySlice::from_slice_unchecked_mut(self.0.spare_capacity_mut()) }
    }

    /// Wrapper around `Vec::splice`.
    #[inline]
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> std::vec::Splice<'_, I::IntoIter>
    where R: std::ops::RangeBounds<usize>, I: IntoIterator<Item = T> {
        self.0.splice(range, replace_with)
    }
}

impl<T: Clone> NonEmptyVec<T> {
    /// Creates a vector from `elem`, copied `n` times.\
    /// Mostly for use in the ne_vec![elem; n] macro.
    #[inline]
    pub fn from_elem(elem: T, n: NonZeroUsize) -> NonEmptyVec<T> {
        NonEmptyVec(std::vec::from_elem(elem, n.get()))
    }

    /// Safe wrapper around `Vec::new_unchecked_mut`, that ensures the vector cannot become empty.
    #[inline]
    pub fn resize(&mut self, new_len: NonZeroUsize, value: T) {
        self.0.resize(new_len.get(), value)
    }

    /// Wrapper around `Vec::extend_from_slice`.
    #[inline]
    pub fn extend_from_slice(&mut self, other: &[T]) {
        self.0.extend_from_slice(other)
    }

    /// Wrapper around `Vec::extend_from_within`.
    #[inline]
    pub fn extend_from_within<R: std::ops::RangeBounds<usize>>(&mut self, src: R) {
        self.0.extend_from_within(src)
    }
}

impl<T: PartialEq> NonEmptyVec<T> {
    /// Wrapper around `Vec::dedup`. This method cannot leave the vector empty, and is as such safe to use.
    #[inline]
    pub fn dedup(&mut self) {
        self.0.dedup()
    }
}

impl<T> std::ops::Deref for NonEmptyVec<T> {
    type Target = NonEmptySlice<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { NonEmptySlice::<T>::from_slice_unchecked(&self.0) }
    }
}

impl<T> std::ops::DerefMut for NonEmptyVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { NonEmptySlice::<T>::from_slice_unchecked_mut(&mut self.0) }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for NonEmptyVec<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

// Cannot implement `from_iter` since iterators may only have one item.

impl<T> IntoIterator for NonEmptyVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a NonEmptyVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut NonEmptyVec<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T> Extend<T> for NonEmptyVec<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }

    // #[inline]
    // fn extend_one(&mut self, item: T) {
    //     self.0.push(item)
    // }

    // #[inline]
    // fn extend_reserve(&mut self, additional: usize) {
    //     self.0.reserve(additional);
    // }
}

impl<'a, T: Copy + 'a> Extend<&'a T> for NonEmptyVec<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }

    // #[inline]
    // fn extend_one(&mut self, item: T) {
    //     self.0.push(item)
    // }

    // #[inline]
    // fn extend_reserve(&mut self, additional: usize) {
    //     self.0.reserve(additional);
    // }
}

impl<T> AsRef<NonEmptyVec<T>> for NonEmptyVec<T> {
    #[inline]
    fn as_ref(&self) -> &NonEmptyVec<T> {
        self
    }
}

impl<T> AsMut<NonEmptyVec<T>> for NonEmptyVec<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut NonEmptyVec<T> {
        self
    }
}

impl<T> AsRef<[T]> for NonEmptyVec<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsRef<NonEmptySlice<T>> for NonEmptyVec<T> {
    #[inline]
    fn as_ref(&self) -> &NonEmptySlice<T> {
        self
    }
}

// AsMut cannot be implemented for already mentioned reasons.

impl<T: Clone> From<&NonEmptySlice<T>> for NonEmptyVec<T> {
    #[inline]
    fn from(s: &NonEmptySlice<T>) -> NonEmptyVec<T> {
        s.to_vec()
    }
}

impl<T: Clone> From<&mut NonEmptySlice<T>> for NonEmptyVec<T> {
    #[inline]
    fn from(s: &mut NonEmptySlice<T>) -> NonEmptyVec<T> {
        s.to_vec()
    }
}

impl<'a, T: Clone> TryFrom<&'a [T]> for NonEmptyVec<T> {
    type Error = &'a [T];

    #[inline]
    fn try_from(s: &[T]) -> Result<NonEmptyVec<T>, &[T]> {
        NonEmptySlice::try_from_slice(s).map(|x| x.to_vec())
    }
}

impl<'a, T: Clone> TryFrom<&'a mut [T]> for NonEmptyVec<T> {
    type Error = &'a mut [T];

    #[inline]
    fn try_from(s: &mut [T]) -> Result<NonEmptyVec<T>, &mut [T]> {
        NonEmptySlice::try_from_slice_mut(s).map(|x| x.to_vec())
    }
}

impl<'a, T: Clone> TryFrom<std::borrow::Cow<'a, [T]>> for NonEmptyVec<T> {
    type Error = std::borrow::Cow<'a, [T]>;

    #[inline]
    fn try_from(s: std::borrow::Cow<'a, [T]>) -> Result<NonEmptyVec<T>, std::borrow::Cow<'a, [T]>> {
        if s.is_empty() { Err(s) }
        else { Ok(NonEmptyVec(s.to_vec())) }
    }
}

impl<T> TryFrom<Box<[T]>> for NonEmptyVec<T> {
    type Error = Box<[T]>;

    #[inline]
    fn try_from(s: Box<[T]>) -> Result<NonEmptyVec<T>, Box<[T]>> {
        if s.is_empty() { Err(s) }
        else { Ok(NonEmptyVec(s.into())) }
    }
}

impl<T> TryFrom<Vec<T>> for NonEmptyVec<T> {
    type Error = Vec<T>;

    #[inline]
    fn try_from(s: Vec<T>) -> Result<NonEmptyVec<T>, Vec<T>> {
        NonEmptyVec::try_from_vec(s)
    }
}

impl From<&str> for NonEmptyVec<u8> {
    #[inline]
    fn from(s: &str) -> NonEmptyVec<u8> {
        NonEmptyVec(From::from(s.as_bytes()))
    }
}

impl<T> From<NonEmptyVec<T>> for Box<[T]> {
    #[inline]
    fn from(s: NonEmptyVec<T>) -> Box<[T]> {
        s.0.into()
    }
}

impl<T> From<NonEmptyVec<T>> for Vec<T> {
    #[inline]
    fn from(s: NonEmptyVec<T>) -> Vec<T> {
        s.0
    }
}



#[cfg(feature = "static_assert_generic")]
use static_assert_generic::static_assert;

#[cfg(feature = "static_assert_generic")]
impl<T: Clone, const N: usize> From<&[T; N]> for NonEmptyVec<T> {
    /// The length of the array is checked at compile time, and as such this method is infalible.
    /// If the length of the array is not 0, a compiler error will be given. This requires a full build and does not show up when running `cargo check`.
    #[inline]
    fn from(s: &[T; N]) -> NonEmptyVec<T> {
        static_assert!((N: usize) N != 0 => "Length of array must be non-zero to create NonEmptyVec.");
        NonEmptyVec(s.to_vec())
    }
}

#[cfg(feature = "static_assert_generic")]
impl<T: Clone, const N: usize> From<&mut [T; N]> for NonEmptyVec<T> {
    /// The length of the array is checked at compile time, and as such this method is infalible.
    /// If the length of the array is not 0, a compiler error will be given. This requires a full build and does not show up when running `cargo check`.
    #[inline]
    fn from(s: &mut [T; N]) -> NonEmptyVec<T> {
        static_assert!((N: usize) N != 0 => "Length of array must be non-zero to create NonEmptyVec.");
        NonEmptyVec(s.to_vec())
    }
}

#[cfg(feature = "static_assert_generic")]
impl<T, const N: usize> From<[T; N]> for NonEmptyVec<T> {
    /// The length of the array is checked at compile time, and as such this method is infalible.
    /// If the length of the array is not 0, a compiler error will be given. This requires a full build and does not show up when running `cargo check`.
    #[inline]
    fn from(s: [T; N]) -> NonEmptyVec<T> {
        static_assert!((N: usize) N != 0 => "Length of array must be non-zero to create NonEmptyVec.");
        NonEmptyVec(s.into())
    }
}

#[cfg(feature = "static_assert_generic")]
impl<T> NonEmptyVec<T> {
    #[inline]
    pub fn from_arr<const N: usize>(arr: [T; N]) -> NonEmptyVec<T> {
        arr.into()
    }
}

#[cfg(feature = "static_assert_generic")]
impl<T, const N: usize> TryFrom<NonEmptyVec<T>> for [T; N] {
    type Error = NonEmptyVec<T>;
    
    /// The length of the array is checked at compile time, and as such this method is infalible.
    /// If the length of the array is not 0, a compiler error will be given. This requires a full build and does not show up when running `cargo check`.
    #[inline]
    fn try_from(s: NonEmptyVec<T>) -> Result<[T; N], NonEmptyVec<T>> {
        static_assert!((N: usize) N != 0 => "Length of array must be non-zero to create NonEmptyVec.");
        if s.is_empty() { Err(s) }
        else {
            match s.0.try_into() {
                Ok(r) => Ok(r),
                Err(r) => Err(NonEmptyVec(r))
            }
        }
    }
}