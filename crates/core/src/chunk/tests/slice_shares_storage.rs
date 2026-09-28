//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Chunk;

// Slicing a chunk shares its original byte storage.
#[test]
fn test() {
  let original = Chunk::from(vec![1, 2, 3, 4, 5]);
  let original_ptr = original.as_ptr();

  let slice = original.slice(1..4);

  assert_eq!(slice.as_ref(), &[2, 3, 4]);
  assert_eq!(slice.as_ptr(), original_ptr.wrapping_add(1));
}
