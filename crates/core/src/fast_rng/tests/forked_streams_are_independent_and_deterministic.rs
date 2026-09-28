//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use rand::Rng as _;
use rand::SeedableRng as _;

use super::super::FastRng;

fn draw_streams() -> ([u64; 2], [u64; 2], [u64; 2]) {
  let mut rng = FastRng::seed_from_u64(1_337);
  let mut first = rng.fork();
  let mut second = rng.fork();

  let root = [rng.next_u64(), rng.next_u64()];
  let first = [first.next_u64(), first.next_u64()];
  let second = [second.next_u64(), second.next_u64()];
  (root, first, second)
}

#[test]
fn test() {
  let (root, first, second) = draw_streams();
  assert_eq!((root, first, second), draw_streams());
  assert_ne!(root, first);
  assert_ne!(root, second);
  assert_ne!(first, second);
}
