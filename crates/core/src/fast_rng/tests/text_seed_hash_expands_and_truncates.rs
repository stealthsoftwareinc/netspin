//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use rand::Rng as _;
use rand::SeedableRng;

use super::super::FastRng;
use super::super::hash_text_into_seed;

#[test]
fn test() {
  const TEXT: &str = "A deterministic seed \u{1F331}";

  let mut short = [0; 7];
  hash_text_into_seed(TEXT, &mut short);
  let mut long = [0; 80];
  hash_text_into_seed(TEXT, &mut long);
  assert_eq!(short, long[..short.len()]);
  assert_ne!(long[..32], long[32..64]);
  let mut duplicate = [0; 80];
  hash_text_into_seed(TEXT, &mut duplicate);
  assert_eq!(long, duplicate);

  let mut seed = <FastRng as SeedableRng>::Seed::default();
  hash_text_into_seed(TEXT, seed.as_mut());
  let mut expected = FastRng::from_seed(seed);
  let mut actual = FastRng::from_seed_text(TEXT);
  for _ in 0..8 {
    assert_eq!(actual.next_u64(), expected.next_u64());
  }
}
