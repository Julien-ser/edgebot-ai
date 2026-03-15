//! Core zero-copy buffer abstractions using `MaybeUninit`
//!
//! This module provides the foundation for safe zero-copy memory operations
//! using Rust's `MaybeUninit` type to handle uninitialized memory safely.
//! All buffers guarantee that uninitialized memory is never read and that
//! alignment requirements are met.

use std::mem::MaybeUninit;
use std::ptr;
use std::slice;

/// Error types that can occur during buffer operations.
#[derive(Debug, thiserror::Error)]
pub enum BufferError {
    /// Buffer is too small for the requested operation.
    #[error("Buffer too small: needed {needed}, have {have}")]
    BufferTooSmall { needed: usize, have: usize },
    /// Attempted to read uninitialized memory.
    #[error("Attempted to read uninitialized memory")]
    UninitializedRead,
    /// Invalid alignment for the buffer operation.
    #[error("Invalid alignment: expected {expected}, got {actual}")]
    InvalidAlignment { expected: usize, actual: usize },
}

/// A zero-copy buffer that safely manages uninitialized memory.
///
/// `ZeroCopyBuffer<T>` provides a safe wrapper around a `MaybeUninit<[T]>` slice,
/// allowing you to:
/// - Allocate uninitialized memory efficiently
/// - Initialize elements safely
/// - Borrow as initialized slices when appropriate
/// - Share memory across different sensor data representations
///
/// # Safety
/// The buffer ensures that:
/// - Uninitialized elements are never read
/// - All methods maintain proper alignment
/// - The buffer's length tracking is accurate
pub struct ZeroCopyBuffer<T> {
    buffer: MaybeUninit<Box<[T]>>,
    initialized_len: usize,
    total_len: usize,
}

impl<T> ZeroCopyBuffer<T> {
    /// Create a new zero-copy buffer with the given total capacity.
    ///
    /// All memory is initially uninitialized.
    pub fn new(total_len: usize) -> Self
    where
        T: MaybeUninit<MaybeUninit<T>>,
    {
        let buffer = vec![MaybeUninit::uninit(); total_len];
        Self {
            buffer: MaybeUninit::new(buffer.into_boxed_slice()),
            initialized_len: 0,
            total_len,
        }
    }

    /// Create a buffer from an existing initialized `Vec<T>`.
    ///
    /// This takes ownership of the vector and marks all elements as initialized.
    pub fn from_vec(vec: Vec<T>) -> Self {
        let total_len = vec.len();
        let buffer = MaybeUninit::new(vec.into_boxed_slice());
        Self {
            buffer,
            initialized_len: total_len,
            total_len,
        }
    }

    /// Get the total capacity of the buffer.
    pub fn total_len(&self) -> usize {
        self.total_len
    }

    /// Get the number of initialized elements.
    pub fn initialized_len(&self) -> usize {
        self.initialized_len
    }

    /// Get the number of uninitialized elements available.
    pub fn uninitialized_len(&self) -> usize {
        self.total_len - self.initialized_len
    }

    /// Initialize the next uninitialized element with the given value.
    ///
    /// Returns `Ok(())` if initialization succeeded, or `Err(value)` if
    /// the buffer was already full.
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.initialized_len >= self.total_len {
            return Err(value);
        }

        // SAFETY: We have space and we're writing to uninitialized memory
        unsafe {
            let ptr = self.buffer.as_mut_ptr().add(self.initialized_len);
            ptr.write(value);
        }
        self.initialized_len += 1;
        Ok(())
    }

    /// Initialize a range of elements with the given slice.
    ///
    /// Returns `Ok(())` on success, or `Err(())` if not enough space.
    pub fn extend_from_slice(&mut self, values: &[T]) -> Result<(), BufferError>
    where
        T: Copy,
    {
        let needed = values.len();
        if self.uninitialized_len() < needed {
            return Err(BufferError::BufferTooSmall {
                needed,
                have: self.uninitialized_len(),
            });
        }

        // SAFETY: We have enough space and T: Copy ensures it's safe to copy
        unsafe {
            let start = self.initialized_len;
            let dest = self.buffer.as_mut_ptr().add(start);
            ptr::copy_nonoverlapping(values.as_ptr(), dest, needed);
        }
        self.initialized_len += needed;
        Ok(())
    }

    /// Get a borrowed slice of the initialized portion of the buffer.
    ///
    /// # Safety
    /// This is safe because we track initialized elements and ensure
    /// we only return initialized data.
    pub fn initialized_slice(&self) -> &[T] {
        // SAFETY: We only access the first `initialized_len` elements
        // which have been marked as initialized.
        unsafe { slice::from_raw_parts(self.buffer.as_ptr(), self.initialized_len) }
    }

    /// Get a mutable slice of the initialized portion of the buffer.
    pub fn initialized_slice_mut(&mut self) -> &mut [T] {
        // SAFETY: Same reasoning as `initialized_slice`
        unsafe { slice::from_raw_parts_mut(self.buffer.as_mut_ptr(), self.initialized_len) }
    }

    /// Get a borrowed slice of the entire buffer (including uninitialized parts).
    ///
    /// # Safety
    /// Caller must not read uninitialized elements.
    pub unsafe fn as_slice(&self) -> &[T] {
        slice::from_raw_parts(self.buffer.as_ptr(), self.total_len)
    }

    /// Get a mutable slice of the entire buffer.
    ///
    /// # Safety
    /// Caller must not read uninitialized elements.
    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        slice::from_raw_parts_mut(self.buffer.as_mut_ptr(), self.total_len)
    }

    /// Consume the buffer and return the fully initialized `Box<[T]>`.
    ///
    /// # Panics
    /// Panics if not all elements are initialized.
    pub fn into_boxed_slice(self) -> Box<[T]> {
        assert_eq!(
            self.initialized_len,
            self.total_len,
            "Cannot consume partially initialized buffer"
        );
        // SAFETY: All elements are initialized
        unsafe { self.buffer.assume_init() }
    }

    /// Consume the buffer and return the underlying `MaybeUninit<Box<[T]>>`.
    pub fn into_inner(self) -> MaybeUninit<Box<[T]>> {
        self.buffer
    }

    /// Clear the buffer, marking all elements as uninitialized.
    ///
    /// # Safety
    /// This drops all initialized elements and resets tracking.
    pub fn clear(&mut self) {
        // Drop all initialized elements
        unsafe {
            let slice = slice::from_raw_parts_mut(self.buffer.as_mut_ptr(), self.initialized_len);
            ptr::drop_in_place(slice);
        }
        self.initialized_len = 0;
    }
}

impl<T> Drop for ZeroCopyBuffer<T> {
    fn drop(&mut self) {
        // Drop any remaining initialized elements
        self.clear();
    }
}

/// A borrowed view into a zero-copy buffer that guarantees initialization.
///
/// `BorrowedBuffer<'a, T>` provides a safe, temporary view into a portion of
/// a `ZeroCopyBuffer` that is guaranteed to be initialized. This allows
/// functions to work with sensor data without taking ownership.
pub struct BorrowedBuffer<'a, T> {
    slice: &'a [T],
}

impl<'a, T> BorrowedBuffer<'a, T> {
    /// Create a new borrowed buffer from a slice (assumes slice is fully initialized).
    pub fn new(slice: &'a [T]) -> Self {
        Self { slice }
    }

    /// Get the length of the buffer.
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }

    /// Get a reference to the underlying slice.
    pub fn as_slice(&self) -> &'a [T] {
        self.slice
    }

    /// Get an iterator over the buffer elements.
    pub fn iter(&self) -> impl Iterator<Item = &'a T> {
        self.slice.iter()
    }

    /// Convert to an `ndarray::ArrayView` for numeric processing.
    ///
    /// # Panics
    /// Panics if `T` is not a numeric type that can be used with ndarray.
    pub fn to_ndarray(&self) -> ndarray::ArrayView<'a, T, ndarray::Ix1>
    where
        T: Copy + 'a,
    {
        ndarray::ArrayView::from(self.slice)
    }

    /// Convert to an `ndarray::ArrayView` with custom dimensionality.
    ///
    /// # Panics
    /// Panics if `shape` does not match the buffer length.
    pub fn to_ndarray_view(&self, shape: &[usize]) -> ndarray::ArrayView<'a, T, ndarray::IxDyn>
    where
        T: Copy + 'a,
    {
        let total_size: usize = shape.iter().product();
        assert_eq!(
            total_size,
            self.slice.len(),
            "Shape mismatch: buffer has {} elements, shape requires {}",
            self.slice.len(),
            total_size
        );

        // This is a simplified version; proper implementation would need
        // to handle different layouts and strides
        let flat = ndarray::Array1::from(self.slice.to_vec());
        flat.into_shape(shape).unwrap().as_view()
    }
}

impl<'a, T> AsRef<[T]> for BorrowedBuffer<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.slice
    }
}

impl<'a, T> std::ops::Deref for BorrowedBuffer<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.slice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_buffer_new() {
        let mut buffer = ZeroCopyBuffer::<i32>::new(10);
        assert_eq!(buffer.total_len(), 10);
        assert_eq!(buffer.initialized_len(), 0);
        assert_eq!(buffer.uninitialized_len(), 10);
    }

    #[test]
    fn test_zero_copy_buffer_push() {
        let mut buffer = ZeroCopyBuffer::<i32>::new(5);
        assert!(buffer.push(1).is_ok());
        assert!(buffer.push(2).is_ok());
        assert_eq!(buffer.initialized_len(), 2);
        assert_eq!(buffer.uninitialized_len(), 3);

        let slice = buffer.initialized_slice();
        assert_eq!(slice, &[1, 2]);
    }

    #[test]
    fn test_zero_copy_buffer_from_vec() {
        let vec = vec![1, 2, 3, 4, 5];
        let buffer = ZeroCopyBuffer::from_vec(vec);
        assert_eq!(buffer.total_len(), 5);
        assert_eq!(buffer.initialized_len(), 5);
        assert_eq!(buffer.initialized_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_zero_copy_buffer_extend() {
        let mut buffer = ZeroCopyBuffer::<i32>::new(10);
        let values = vec![1, 2, 3];
        assert!(buffer.extend_from_slice(&values).is_ok());
        assert_eq!(buffer.initialized_len(), 3);
        assert_eq!(buffer.initialized_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_zero_copy_buffer_extend_fails_when_full() {
        let mut buffer = ZeroCopyBuffer::<i32>::new(3);
        buffer.push(1).unwrap();
        buffer.push(2).unwrap();
        buffer.push(3).unwrap();

        let values = vec![4, 5];
        let result = buffer.extend_from_slice(&values);
        assert!(matches!(result, Err(BufferError::BufferTooSmall { .. })));
    }

    #[test]
    fn test_borrowed_buffer() {
        let vec = vec![1, 2, 3, 4, 5];
        let borrowed = BorrowedBuffer::new(&vec);
        assert_eq!(borrowed.len(), 5);
        assert_eq!(borrowed.as_slice(), &[1, 2, 3, 4, 5]);
        assert!(iter::Iterator::all(borrowed.iter(), |&x| x > 0));
    }

    #[test]
    fn test_borrowed_buffer_to_ndarray() {
        let vec = vec![1.0, 2.0, 3.0, 4.0];
        let borrowed = BorrowedBuffer::new(&vec);
        let array = borrowed.to_ndarray();
        assert_eq!(array.len(), 4);
        assert_eq!(array.sum(), 10.0);
    }

    #[test]
    #[should_panic(expected = "Shape mismatch")]
    fn test_borrowed_buffer_to_ndarray_view_wrong_shape() {
        let vec = vec![1, 2, 3, 4];
        let borrowed = BorrowedBuffer::new(&vec);
        let _ = borrowed.to_ndarray_view(&[2, 3]); // 6 elements, mismatch
    }
}
