//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::convert::Infallible;

use rand::SeedableRng;
use rand::TryRng;
use rand::rngs::Xoshiro256PlusPlus;
use sha2::Digest as _;
use sha2::Sha256;

fn hash_text_into_seed(text: &str, seed: &mut [u8]) {
  let mut remaining = seed;
  let mut counter = 0_u64;
  while !remaining.is_empty() {
    let digest = Sha256::new()
      .chain_update(counter.to_le_bytes())
      .chain_update(text.as_bytes())
      .finalize();
    let length = remaining.len().min(digest.len());
    let (block, rest) = remaining.split_at_mut(length);
    block.copy_from_slice(&digest[..length]);
    remaining = rest;
    if !remaining.is_empty() {
      counter += 1;
    }
  }
}

/// A fast RNG that's not necessarily cryptographically secure.
pub struct FastRng {
  rng: Xoshiro256PlusPlus,
}

impl FastRng {
  /// Creates an RNG deterministically seeded from `text`.
  #[must_use]
  pub fn from_seed_text(text: &str) -> Self {
    let mut seed = <Self as SeedableRng>::Seed::default();
    hash_text_into_seed(text, seed.as_mut());
    Self::from_seed(seed)
  }
}

impl SeedableRng for FastRng {
  type Seed = <Xoshiro256PlusPlus as SeedableRng>::Seed;

  fn from_seed(seed: Self::Seed) -> Self {
    Self {
      rng: Xoshiro256PlusPlus::from_seed(seed),
    }
  }
}

impl TryRng for FastRng {
  type Error = Infallible;

  fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
    self.rng.try_next_u32()
  }

  fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
    self.rng.try_next_u64()
  }

  fn try_fill_bytes(
    &mut self,
    dst: &mut [u8],
  ) -> Result<(), Self::Error> {
    self.rng.try_fill_bytes(dst)
  }
}

#[cfg(test)]
mod tests;
