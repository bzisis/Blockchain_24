use crate::SmallVec8;
use ssz::{Decode, Encode};
use ssz_derive::{Decode, Encode};
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ops::Range;

/// Errors that can occur during operations on the `CacheArena`.
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    /// The specified allocation ID does not exist.
    UnknownAllocId(usize),
    /// Overflow occurred while calculating an offset.
    OffsetOverflow,
    /// Underflow occurred while calculating an offset.
    OffsetUnderflow,
    /// The specified range is invalid.
    RangeOverFlow,
}

/// A memory arena that provides a single contiguous memory allocation
/// from which smaller allocations can be produced. It aims to reduce memory
/// fragmentation by storing multiple `Vec<T>`-like objects contiguously on the heap.
///
/// All allocations are stored in one large `Vec`, so resizing any allocation
/// will move all items to the right of that allocation.
#[derive(Debug, PartialEq, Clone, Default, Encode, Decode)]
pub struct CacheArena<T: Encode + Decode> {
    /// The backing array, storing cached values.
    backing: Vec<T>,
    /// A list of offsets indicating the start of each allocation.
    offsets: Vec<usize>,
}

impl<T: Encode + Decode> CacheArena<T> {
    /// Creates a new `CacheArena` with a backing array of the given `capacity`.
    ///
    /// # Parameters
    ///
    /// - `capacity`: The initial capacity of the backing array.
    ///
    /// # Returns
    ///
    /// A new `CacheArena` instance.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            backing: Vec::with_capacity(capacity),
            offsets: vec![],
        }
    }

    /// Allocates a new zero-length allocation at the end of the backing array.
    ///
    /// # Returns
    ///
    /// A `CacheArenaAllocation` instance representing the new allocation.
    pub fn alloc(&mut self) -> CacheArenaAllocation<T> {
        let alloc_id = self.offsets.len();
        self.offsets.push(self.backing.len());

        CacheArenaAllocation {
            alloc_id,
            _phantom: PhantomData,
        }
    }

    /// Updates the offsets to reflect an allocation growing in size.
    ///
    /// # Parameters
    ///
    /// - `alloc_id`: The ID of the allocation to grow.
    /// - `grow_by`: The amount to grow the allocation by.
    ///
    /// # Returns
    ///
    /// A result indicating success or the specific error.
    fn grow(&mut self, alloc_id: usize, grow_by: usize) -> Result<(), Error> {
        if alloc_id < self.offsets.len() {
            self.offsets
                .iter_mut()
                .skip(alloc_id + 1)
                .try_for_each(|offset| {
                    *offset = offset.checked_add(grow_by).ok_or(Error::OffsetOverflow)?;

                    Ok(())
                })
        } else {
            Err(Error::UnknownAllocId(alloc_id))
        }
    }

    /// Updates the offsets to reflect an allocation shrinking in size.
    ///
    /// # Parameters
    ///
    /// - `alloc_id`: The ID of the allocation to shrink.
    /// - `shrink_by`: The amount to shrink the allocation by.
    ///
    /// # Returns
    ///
    /// A result indicating success or the specific error.
    fn shrink(&mut self, alloc_id: usize, shrink_by: usize) -> Result<(), Error> {
        if alloc_id < self.offsets.len() {
            self.offsets
                .iter_mut()
                .skip(alloc_id + 1)
                .try_for_each(|offset| {
                    *offset = offset
                        .checked_sub(shrink_by)
                        .ok_or(Error::OffsetUnderflow)?;

                    Ok(())
                })
        } else {
            Err(Error::UnknownAllocId(alloc_id))
        }
    }

    /// Replaces a range of elements in the specified allocation with new elements.
    /// Similar to `Vec::splice`, but the range is relative to the allocation and not returned.
    ///
    /// # Parameters
    ///
    /// - `alloc_id`: The ID of the allocation to splice.
    /// - `range`: The range of elements to replace, relative to the allocation.
    /// - `replace_with`: An iterator over the new elements.
    ///
    /// # Returns
    ///
    /// A result indicating success or the specific error.
    fn splice_forgetful<I: IntoIterator<Item = T>>(
        &mut self,
        alloc_id: usize,
        range: Range<usize>,
        replace_with: I,
    ) -> Result<(), Error> {
        let offset = *self
            .offsets
            .get(alloc_id)
            .ok_or(Error::UnknownAllocId(alloc_id))?;
        let start = range
            .start
            .checked_add(offset)
            .ok_or(Error::RangeOverFlow)?;
        let end = range.end.checked_add(offset).ok_or(Error::RangeOverFlow)?;

        let prev_len = self.backing.len();

        self.backing.splice(start..end, replace_with);

        match prev_len.cmp(&self.backing.len()) {
            Ordering::Greater => self.shrink(alloc_id, prev_len - self.backing.len())?,
            Ordering::Less => self.grow(alloc_id, self.backing.len() - prev_len)?,
            Ordering::Equal => {}
        }

        Ok(())
    }

    /// Returns the length of the specified allocation.
    ///
    /// # Parameters
    ///
    /// - `alloc_id`: The ID of the allocation.
    ///
    /// # Returns
    ///
    /// The length of the allocation, or an error if the allocation ID is unknown.
    fn len(&self, alloc_id: usize) -> Result<usize, Error> {
        let start = self
            .offsets
            .get(alloc_id)
            .ok_or(Error::UnknownAllocId(alloc_id))?;
        let end = self
            .offsets
            .get(alloc_id + 1)
            .copied()
            .unwrap_or(self.backing.len());

        Ok(end - start)
    }

    /// Returns a reference to the value at position `i` within the specified allocation.
    ///
    /// # Parameters
    ///
    /// - `alloc_id`: The ID of the allocation.
    /// - `i`: The position within the allocation.
    ///
    /// # Returns
    ///
    /// An optional reference to the value, or an error if the allocation ID is unknown.
    fn get(&self, alloc_id: usize, i: usize) -> Result<Option<&T>, Error> {
        if i < self.len(alloc_id)? {
            let offset = self
                .offsets
                .get(alloc_id)
                .ok_or(Error::UnknownAllocId(alloc_id))?;
            Ok(self.backing.get(i + offset))
        } else {
            Ok(None)
        }
    }

    /// Returns a mutable reference to the value at position `i` within the specified allocation.
    ///
    /// # Parameters
    ///
    /// - `alloc_id`: The ID of the allocation.
    /// - `i`: The position within the allocation.
    ///
    /// # Returns
    ///
    /// An optional mutable reference to the value, or an error if the allocation ID is unknown.
    fn get_mut(&mut self, alloc_id: usize, i: usize) -> Result<Option<&mut T>, Error> {
        if i < self.len(alloc_id)? {
            let offset = self
                .offsets
                .get(alloc_id)
                .ok_or(Error::UnknownAllocId(alloc_id))?;
            Ok(self.backing.get_mut(i + offset))
        } else {
            Ok(None)
        }
    }

    /// Returns the range in the backing array occupied by the specified allocation.
    ///
    /// # Parameters
    ///
    /// - `alloc_id`: The ID of the allocation.
    ///
    /// # Returns
    ///
    /// The range of the allocation, or an error if the allocation ID is unknown.
    fn range(&self, alloc_id: usize) -> Result<Range<usize>, Error> {
        let start = *self
            .offsets
            .get(alloc_id)
            .ok_or(Error::UnknownAllocId(alloc_id))?;
        let end = self
            .offsets
            .get(alloc_id + 1)
            .copied()
            .unwrap_or(self.backing.len());

        Ok(start..end)
    }

    /// Returns an iterator over the values in the specified allocation.
    ///
    /// # Parameters
    ///
    /// - `alloc_id`: The ID of the allocation.
    ///
    /// # Returns
    ///
    /// An iterator over the values, or an error if the allocation ID is unknown.
    fn iter(&self, alloc_id: usize) -> Result<impl Iterator<Item = &T>, Error> {
        Ok(self.backing[self.range(alloc_id)?].iter())
    }

    /// Returns a mutable iterator over the values in the specified allocation.
    ///
    /// # Parameters
    ///
    /// - `alloc_id`: The ID of the allocation.
    ///
    /// # Returns
    ///
    /// A mutable iterator over the values, or an error if the allocation ID is unknown.
    fn iter_mut(&mut self, alloc_id: usize) -> Result<impl Iterator<Item = &mut T>, Error> {
        let range = self.range(alloc_id)?;
       
