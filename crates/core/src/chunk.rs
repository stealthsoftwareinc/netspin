//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::ops::Deref;
use core::ops::Range;

use bytes::Bytes;

/// An immutable, cheaply cloneable and splittable chunk of bytes.
///
/// Cloning and splitting share the underlying storage.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Chunk(Bytes);

impl Chunk {
  /// Returns a pointer to the first byte in the chunk.
  #[must_use]
  pub fn as_ptr(&self) -> *const u8 {
    self.0.as_ptr()
  }

  /// Returns whether the chunk is empty.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  /// Returns the number of bytes in the chunk.
  #[must_use]
  pub fn len(&self) -> usize {
    self.0.len()
  }

  /// Returns a chunk containing the requested byte range.
  ///
  /// The returned chunk shares the underlying storage with `self`.
  #[must_use]
  pub fn slice(&self, range: Range<usize>) -> Self {
    assert!(
      range.start <= range.end && range.end <= self.len(),
      "netspin_core::Chunk::slice(): The range must fit in the chunk.",
    );
    Self(self.0.slice(range))
  }

  /// Splits the chunk at `at`, returning the bytes after the split.
  ///
  /// This retains the bytes before the split in `self` and shares the
  /// underlying storage with the returned chunk.
  #[must_use]
  pub fn split_off(&mut self, at: usize) -> Self {
    assert!(
      at <= self.len(),
      "netspin_core::Chunk::split_off(): The split index exceeds the \
       chunk length.",
    );
    Self(self.0.split_off(at))
  }
}

impl AsRef<[u8]> for Chunk {
  fn as_ref(&self) -> &[u8] {
    self.0.as_ref()
  }
}

impl Deref for Chunk {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    self.0.deref()
  }
}

impl From<Chunk> for Vec<u8> {
  fn from(value: Chunk) -> Self {
    value.0.into()
  }
}

impl From<Vec<u8>> for Chunk {
  fn from(value: Vec<u8>) -> Self {
    Self(value.into())
  }
}

impl PartialEq<Vec<u8>> for Chunk {
  fn eq(&self, other: &Vec<u8>) -> bool {
    self.as_ref() == other.as_slice()
  }
}

#[cfg(test)]
mod tests;
