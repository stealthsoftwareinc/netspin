//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Chunk;

// Splitting a chunk shares its original byte storage.
#[test]
fn test() {
  let mut first = Chunk::from(vec![1, 2, 3, 4, 5]);
  let first_ptr = first.as_ptr();
  let second_ptr = first_ptr.wrapping_add(3);

  let second = first.split_off(3);

  assert_eq!(first.as_ref(), &[1, 2, 3]);
  assert_eq!(second.as_ref(), &[4, 5]);
  assert_eq!(first.as_ptr(), first_ptr);
  assert_eq!(second.as_ptr(), second_ptr);
}
