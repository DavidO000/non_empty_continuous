use std::num::NonZeroUsize;

use smallvec::*;

use crate::non_empty_slice::*;

/// The easiest way to create a non-empty smallvec.
/// An error will be raised if no elements are porvided.
/// Repeating syntax requires a `NonZeroUsize`.
/// 
/// # Examples
/// ```
/// let non_empty_smallvec_from_macro = ne_smallvec![99, 98, 97];
/// let non_empty_smallvec_from_macro2 = ne_smallvec![0; std::num::NonZeroUsize::new(100).unwrap()];
/// let _ = ne_smallvec![]; // Error: Cannot make an empty NonEmptySmallVec
/// ```
#[macro_export]
macro_rules! ne_smallvec {
    ($($item: expr),+ $(,)?) => {
        unsafe { $crate::NonEmptySmallVec::from_buf_unchecked([$($item),+]) }
    };
    ($item: expr; $amount: expr) => {
        $crate::NonEmptySmallVec::from_elem($item, $amount)
    };
    () => {
        compile_error!("Cannot make an empty NonEmptySmallVec");
    }
}

/// A wrapper around `SmallVec` that ensures it's not empty.
/// 
/// Getting direct mutable acces to the inner smallvec is not allowed, 
/// since that way setting the size of the vector to 0 becomes possible.
/// As such, many methods that mutate the inner vector are re-implemented.
#[repr(transparent)]
pub struct NonEmptySmallVec<A: Array>(pub(crate) SmallVec<A>);

impl<A: Array> NonEmptySmallVec<A> {
    /// Creates a new NonEmptySmallVec, with precisely one element inside of it. 
    /// If you're starting off with more than one item, consider using 
    /// `NonEmptySmallVec::with_capacity`, `NonEmptyVec::from(array)`, or the `ne_smallvec!` macro.
    #[inline]
    pub fn new(item: A::Item) -> NonEmptySmallVec<A> {
        NonEmptySmallVec(smallvec![item])
    }

    /// Creates a new NonEmptySmallVec, with precisely one element inside of it, and a 
    /// stated capacity. The capacity works the same way as in `SmallVec::with_capacity`.
    #[inline]
    pub fn with_capacity(item: A::Item, capacity: usize) -> NonEmptySmallVec<A> {
        let mut vec = SmallVec::with_capacity(capacity);
        vec.push(item);
        NonEmptySmallVec(vec)
    }

    /// Safely turns a `SmallVec` into a `NonEmptySmallVec` if the smallvec is not empty, 
    /// otherwise an `Err` containing the original smallvec is returned.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let non_empty_smallvec: SmallVec<[i32; 10]> = smallvec![1, 2, 3];
    /// assert_eq!(NonEmptySmallVec::try_from_smallvec(non_empty_smallvec), Ok(NonEmptySmallVec::from_buf([1, 2, 3]));
    /// 
    /// let empty_vec: SmallVec<[i32; 10]> = smallvec![];
    /// assert_eq!(NonEmptySmallVec::try_from_vec(empty_vec), Err(smallvec![]));
    /// ```
    #[inline]
    pub fn try_from_smallvec(smallvec: SmallVec<A>) -> Result<NonEmptySmallVec<A>, SmallVec<A>> {
        if smallvec.is_empty() { Err(smallvec) }
        else { Ok(NonEmptySmallVec(smallvec)) }
    }

    /// If `smallvec` is empty, the vector is not modified and `None` is returned. 
    /// Otherwise, `smallvec`'s items are moved to the new `NonEmptySmallVec` and `smallvec` is emptied.
    #[inline]
    pub fn try_from_smallvec_ref_mut(smallvec: &mut SmallVec<A>) -> Option<NonEmptySmallVec<A>> {
        if smallvec.is_empty() { None }
        else {
            let mut non_empty_vec = NonEmptySmallVec(SmallVec::new());
            std::mem::swap(smallvec, &mut non_empty_vec.0);
            Some(non_empty_vec)
        }
    }

    /// # Safety
    /// `smallvec` must not be empty.
    #[inline]
    pub unsafe fn from_smallvec_unchecked(smallvec: SmallVec<A>) -> NonEmptySmallVec<A> {
        NonEmptySmallVec(smallvec)
    }

    /// For use if the `static_assert_generic` feature is not used. It is highly encouraged that `from_buf` is used instead.
    /// # Safety
    /// The length of the array must not be 0.
    #[inline]
    pub unsafe fn from_buf_unchecked(buf: A) -> NonEmptySmallVec<A> {
        
        NonEmptySmallVec(SmallVec::from_buf(buf))
    }

    /// Get a read-only reference to the underlying `SmallVec`.
    #[inline]
    pub fn get_smallvec(&self) -> &SmallVec<A> {
        &self.0
    }

    /// Moves the `SmallVec` out of the object.
    #[inline]
    pub fn into_smallvec(self) -> SmallVec<A> {
        self.0
    }

    /// Exact wrapper for `SmallVec::into_vec`, exists only for convenience.\
    /// Same as self.get_smallvec().spilled().
    #[inline]
    pub fn into_vec(self) -> Vec<A::Item> {
        self.0.into_vec()
    }

    /// Exact wrapper for `SmallVec::into_boxed_slice`, exists only for convenience.\
    /// Same as self.get_smallvec().spilled().
    #[inline]
    pub fn into_boxed_slice(self) -> Box<[A::Item]> {
        self.0.into_boxed_slice()
    }

    /// Exact wrapper for `SmallVec::into_inner`, exists only for convenience.\
    /// Same as self.get_smallvec().spilled().
    #[inline]
    pub fn into_inner(self) -> Result<A, smallvec::SmallVec<A>> {
        self.0.into_inner()
    }

    /// Returns the vector's capacity, guaranteeing non-zero-ness.\
    #[inline]
    pub fn capacity(&self) -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(self.0.capacity()) }
    }

    /// Exact wrapper for `SmallVec::spilled`, exists only for convenience.\
    /// Same as self.get_smallvec().spilled().
    #[inline]
    pub fn spilled(&self) -> bool {
        self.0.spilled()
    }

    /// Wrapper for `SmallVec::setlen`. `new_len` still needs to be non-zero.
    /// # Safety
    /// Same requirements as `SmallVec::setlen` apply.
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: NonZeroUsize) {
        self.0.set_len(new_len.get())
    }

    /// Safe wrapper around  `SmallVec::drain`.\
    /// This method returns `None` and does not remove any elements if it coveres the whole vector.
    #[inline]
    pub fn drain<R: std::ops::RangeBounds<usize>>(&mut self, range: R) -> Option<smallvec::Drain<'_, A>> {
        if range.contains(&0) && range.contains(&(self.len().get() - 1)) {
            None
        } else {
            Some(self.0.drain(range))
        }
    }

    /// Unsafe wrapper around `SmallVec::drain.`\
    /// For a safe version of this method, use `NonEmptySmallVec::drain`.
    /// # Safety
    /// `range` must not take up the entire vector.
    #[inline]
    pub unsafe fn drain_unchecked<R: std::ops::RangeBounds<usize>>(&mut self, range: R) -> smallvec::Drain<'_, A> {
        self.0.drain(range)
    }

    
    // Not sure if this is worth adding.

    // /// Unsafe wrapper around `SmallVec::drain_filter.`\
    // /// There is no safe counterpart to this method.
    // /// # Safety
    // /// The closure must return `true` at least once. This is very hard to prove 
    // /// statically, and as such is highly discouraged if other options are available.
    // #[inline]
    // pub unsafe fn drain_filter_unchecked<F: FnMut(A::Item) -> bool>(&mut self) {
    //     self.0.drain_filter()
    // }


    /// Wrapper for `SmallVec::push`, reimplemented since a direct mutable reference cannot be given to the underlying vector.
    #[inline]
    pub fn push(&mut self, item: A::Item) {
        self.0.push(item)
    }

    /// Wrapper around `SmallVec::insert`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    /// Inserts the element at the given index, shifting items as needed.
    #[inline]
    pub fn insert(&mut self, index: usize, element: A::Item) {
        self.0.insert(index, element)
    }

    /// Wrapper around `SmallVec::insert`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    /// Inserts the element at the given index, shifting items as needed.
    #[inline]
    pub fn insert_many<I: IntoIterator<Item = A::Item>>(&mut self, index: usize, iterable: I) {
        self.0.insert_many(index, iterable)
    }

    /// Safe wrapper around `SmallVec::pop`.\
    /// Returns `None` and does not pop the element if this would cause the smallvec to become empty.
    #[inline]
    pub fn pop(&mut self) -> Option<A::Item> {
        if self.has_just_1_element() {
            None
        } else {
            self.0.pop()
        }
    }

    /// Wrapper around `SmallVec::append`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    /// This method empties `other`, meaning it cannot be a `NonEmptyVec`.
    #[inline]
    pub fn append_smallvec(&mut self, other: &mut SmallVec<A>) {
        self.0.append(other)
    }

    /// Wrapper around `SmallVec::reserve`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    /// Wrapper around `SmallVec::try_reserve`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), CollectionAllocErr> {
        self.0.try_reserve(additional)
    }

    /// Wrapper around `SmallVec::reserve_exact`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional)
    }

    /// Wrapper around `SmallVec::try_reserve_exact`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), CollectionAllocErr> {
        self.0.try_reserve_exact(additional)
    }

    /// Wrapper around `SmallVec::shrink_to_fit`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// Safe wrapper around `SmallVec::truncate`.
    #[inline]
    pub fn truncate(&mut self, len: NonZeroUsize) {
        self.0.truncate(len.get())
    }

    /// Gets a reference this vector's slice, preserving non-emptyness guarantees.\
    /// This type implements `Deref<Target = NonEmptySlice>`, consider just borrowing instead.
    #[inline]
    pub fn as_slice(&self) -> &NonEmptySlice<A::Item> {
        self
    }

    /// Gets a reference this vector's slice, preserving non-emptyness guarantees.\
    /// This type implements `Deref<Target = NonEmptySlice>`, consider just borrowing instead.
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut NonEmptySlice<A::Item> {
        self
    }

    /// Wrapper around `SmallVec::swap_remove`.\
    /// This method ensures that it won't cause the vector to become empty by not allowing the first element to be removed.
    /// For a mehod that accepts any index, use `NonEmptySmallVec::try_swap_remove`.
    #[inline]
    pub fn swap_remove(&mut self, index: NonZeroUsize) -> A::Item {
        self.0.swap_remove(index.get())
    }

    /// Safe wrapper around `SmallVec::swap_remove`.\
    /// Returns `None` if this would cause the vector to become empty.
    /// Otherwise, moves out the element at `index` and replaces it with the last element in the vector.
    #[inline]
    pub fn try_swap_remove(&mut self, index: usize) -> Option<A::Item> {
        if self.has_just_1_element() {
            None
        } else {
            Some(self.0.swap_remove(index))
        }
    }

    /// Unsafe wrapper around `SmallVec::swap_remove`.\
    /// For a safe version of this method, use `NonEmptySmallVec::try_swap_remove`.
    /// # Safety
    /// Running this must not cause the vector to become empty.
    #[inline]
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> A::Item {
        self.0.swap_remove(index)
    }

    /// Safe wrapper around `SmallVec::remove`.\
    /// Returns `None` if this would cause the vector to become empty.
    #[inline]
    pub fn try_remove(&mut self, index: usize) -> Option<A::Item> {
        if self.has_just_1_element(){
            None
        } else {
            Some(self.0.remove(index))
        }
    }

    /// Unsafe wrapper around `SmallVec::remove`.\
    /// For a safe version of this method, use `NonEmptySmallVec::try_remove`.
    /// # Safety
    /// Running this must not cause the vector to become empty.
    #[inline]
    pub unsafe fn remove_unchecked(&mut self, index: usize) -> A::Item {
        self.0.remove(index)
    }

    /// Wrapper around `SmallVec::dedup_by_key`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    /// Dedup cannot leave the vector empty so this method is safe to use.
    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, key: F) where F: FnMut(&mut A::Item) -> K, K: PartialEq {
        self.0.dedup_by_key(key)
    }

    /// Wrapper around `SmallVec::dedup_by`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    /// Dedup cannot leave the vector empty so this method is safe to use.
    #[inline]
    pub fn dedup_by<F>(&mut self, same_bucket: F) where F: FnMut(&mut A::Item, &mut A::Item) -> bool {
        self.0.dedup_by(same_bucket)
    }

    /// Safe wrapper around `SmallVec::resize_with`, that ensures the vector cannot become empty.
    #[inline]
    pub fn resize_with<F: FnMut() -> A::Item>(&mut self, new_len: NonZeroUsize, f: F) {
        self.0.resize_with(new_len.get(), f)
    }

    /// Creates a `NonEmptySmallVec` from raw parts, ensuring that its length and capacity are above 0.
    /// # Safety
    /// This comes with the same requirements as `NonEmptyVec::from_raw_parts`
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut A::Item, length: NonZeroUsize, capacity: NonZeroUsize) -> NonEmptySmallVec<A> {
        NonEmptySmallVec(SmallVec::from_raw_parts(ptr, length.get(), capacity.get()))
    }

    /// Safe wrapper around `SmallVec::grow`.
    #[inline]
    pub fn grow(&mut self, new_cap: NonZeroUsize) {
        self.0.grow(new_cap.get())
    }

    /// Safe wrapper around `SmallVec::try_grow`.
    #[inline]
    pub fn try_grow(&mut self, new_cap: NonZeroUsize) -> Result<(), CollectionAllocErr> {
        self.0.try_grow(new_cap.get())
    }
}



impl<A: Array> NonEmptySmallVec<A> where A::Item: PartialEq {
    /// Wrapper around `SmallVec::dedup`. This method cannot leave the vector empty, and is as such safe to use.
    #[inline]
    pub fn dedup(&mut self) {
        self.0.dedup()
    }
}



impl<A: Array> NonEmptySmallVec<A> where A::Item: Copy {
    /// Safe wrapper for `SmallVec::from_slice`, guaranteeing non-emptyness.
    #[inline]
    pub fn from_slice(slice: &NonEmptySlice<A::Item>) -> NonEmptySmallVec<A> {
        NonEmptySmallVec(SmallVec::from_slice(slice))
    }

    /// Wrapper for `SmallVec::insert_from_slice`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    #[inline]
    pub fn insert_from_slice(&mut self, index: usize, slice: &[A::Item]) {
        self.0.insert_from_slice(index, slice)
    }

    /// Wrapper for `SmallVec::extend_from_slice`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    #[inline]
    pub fn extend_from_slice(&mut self, slice: &[A::Item]) {
        self.0.extend_from_slice(slice)
    }
}



impl<A: Array> NonEmptySmallVec<A> where A::Item: Clone {
    /// Safe wrapper for `SmallVec::resize`, reimplemented since a direct mutable reference cannot be given to the underlying vector.\
    #[inline]
    pub fn resize(&mut self, len: NonZeroUsize, value: A::Item) {
        self.0.resize(len.get(), value)
    }

    /// Creates a vector from `elem`, copied `n` times.\
    /// Mostly for use in the ne_vec![elem; n] macro.
    #[inline]
    pub fn from_elem(elem: A::Item, n: NonZeroUsize) -> NonEmptySmallVec<A> {
        NonEmptySmallVec(SmallVec::from_elem(elem, n.get()))
    }
}



impl<A: Array> std::ops::Deref for NonEmptySmallVec<A> {
    type Target = NonEmptySlice<A::Item>;
    
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { NonEmptySlice::from_slice_unchecked(&self.0) }
    }
}

impl<A: Array> std::ops::DerefMut for NonEmptySmallVec<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { NonEmptySlice::from_slice_unchecked_mut(&mut self.0) }
    }
}



impl<A: Array> Extend<A::Item> for NonEmptySmallVec<A> {
    fn extend<T: IntoIterator<Item = A::Item>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}


impl<A: Array> std::fmt::Debug for NonEmptySmallVec<A> where A::Item: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<A: Array> Clone for NonEmptySmallVec<A> where A::Item: Clone {
    fn clone(&self) -> Self {
        NonEmptySmallVec(self.0.clone())
    }

    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0)
    }
}

impl<A: Array, B: Array> PartialEq<NonEmptySmallVec<A>> for NonEmptySmallVec<B> where B::Item: PartialEq<A::Item> {
    fn eq(&self, other: &NonEmptySmallVec<A>) -> bool {
        self.0 == other.0
    }
}

impl<A: Array> Eq for NonEmptySmallVec<A> where A::Item: Eq {}

impl<A: Array> PartialOrd for NonEmptySmallVec<A> where A::Item: PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<A: Array> Ord for NonEmptySmallVec<A> where A::Item: Ord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<A: Array> std::hash::Hash for NonEmptySmallVec<A> where A::Item: std::hash::Hash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<A: Array> std::iter::IntoIterator for NonEmptySmallVec<A> {
    type IntoIter = smallvec::IntoIter<A>;
    type Item = A::Item;
    
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// `iter` and `iter_mut` for `NonEmptySmallVec` are not implemented since it dereferences to `NonEmptySlice` anyway.



impl<A: Array> AsRef<[A::Item]> for NonEmptySmallVec<A> {
    #[inline]
    fn as_ref(&self) -> &[A::Item] {
        self
    }
}

impl<A: Array> AsMut<[A::Item]> for NonEmptySmallVec<A> {
    #[inline]
    fn as_mut(&mut self) -> &mut [A::Item] {
        self
    }
}

impl<A: Array> core::borrow::Borrow<[A::Item]> for NonEmptySmallVec<A> {
    #[inline]
    fn borrow(&self) -> &[A::Item] {
        self
    }
}

impl<A: Array> core::borrow::BorrowMut<[A::Item]> for NonEmptySmallVec<A> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [A::Item] {
        self
    }
}



impl<A: Array> AsRef<NonEmptySlice<A::Item>> for NonEmptySmallVec<A> {
    #[inline]
    fn as_ref(&self) -> &NonEmptySlice<A::Item> {
        self
    }
}

impl<A: Array> AsMut<NonEmptySlice<A::Item>> for NonEmptySmallVec<A> {
    #[inline]
    fn as_mut(&mut self) -> &mut NonEmptySlice<A::Item> {
        self
    }
}

impl<A: Array> core::borrow::Borrow<NonEmptySlice<A::Item>> for NonEmptySmallVec<A> {
    #[inline]
    fn borrow(&self) -> &NonEmptySlice<A::Item> {
        self
    }
}

impl<A: Array> core::borrow::BorrowMut<NonEmptySlice<A::Item>> for NonEmptySmallVec<A> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut NonEmptySlice<A::Item> {
        self
    }
}



impl<A: Array> TryFrom<SmallVec<A>> for NonEmptySmallVec<A> {
    type Error = SmallVec<A>;

    #[inline]
    fn try_from(small_vec: SmallVec<A>) -> Result<Self, Self::Error> {
        NonEmptySmallVec::try_from_smallvec(small_vec)
    }
}

impl<A: Array> From<NonEmptySmallVec<A>> for SmallVec<A> {
    #[inline]
    fn from(non_empty_small_vec: NonEmptySmallVec<A>) -> Self {
        non_empty_small_vec.0
    }
}

impl<T, const N: usize>  NonEmptySmallVec<[T; N]> {
    /// Wrapper around `SmallVec::from_buf`.\
    /// The length of the array is checked at compile time, and as such this method is infalible.
    /// If the length of the array is not 0, a compiler error will be given. This requires a full build and does not show up when running `cargo check`.
    #[inline]
    pub fn from_buf(buf: [T; N]) -> NonEmptySmallVec<[T; N]> {
        const { assert!(N > 0, "Length of array must be non-zero to create NonEmptySmallVec."); }
        NonEmptySmallVec(SmallVec::from_buf(buf))
    }

    /// Safe wrapper around `SmallVec::from_buf_and_len`.\
    /// The length of the array is checked at compile time, and as such this method is infalible.
    /// If the length of the array is not 0, a compiler error will be given. This requires a full build and does not show up when running `cargo check`.
    #[inline]
    pub fn from_buf_and_len(buf: [T; N], len: NonZeroUsize) -> NonEmptySmallVec<[T; N]> {
        const { assert!(N > 0, "Length of array must be non-zero to create NonEmptySmallVec."); }
        NonEmptySmallVec(SmallVec::from_buf_and_len(buf, len.get()))
    }

    /// Wrapper around `SmallVec::from_buf_and_len_unchecked`.\
    /// The length of the array is checked at compile time, and as such this method is infalible.
    /// If the length of the array is not 0, a compiler error will be given. This requires a full build and does not show up when running `cargo check`.
    #[inline]
    pub unsafe fn from_buf_and_len_unchecked(buf: core::mem::MaybeUninit<[T; N]>, len: NonZeroUsize) -> NonEmptySmallVec<[T; N]> {
        const { assert!(N > 0, "Length of array must be non-zero to create NonEmptySmallVec."); }
        NonEmptySmallVec(SmallVec::from_buf_and_len_unchecked(buf, len.get()))
    }
}

impl<T, const N: usize> From<[T; N]> for NonEmptySmallVec<[T; N]> {
    fn from(buf: [T; N]) -> Self {
        NonEmptySmallVec(SmallVec::from_buf(buf))
    }
}





#[cfg(feature = "dep:smallvec/write")]
impl<A: Array<Item = u8>> std::io::Write for NonEmptySmallVec<A> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}