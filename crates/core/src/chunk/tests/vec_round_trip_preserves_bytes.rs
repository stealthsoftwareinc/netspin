//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Chunk;

// Converting a vector through a chunk preserves its bytes.
#[test]
fn test() {
  let expected = vec![1, 2, 3];

  let actual = Vec::from(Chunk::from(expected.clone()));

  assert_eq!(actual, expected);
}
