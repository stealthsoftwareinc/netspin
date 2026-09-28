//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use rand::RngExt;

use crate::Task;
use crate::loss::LossModel;

/// A loss model that drops each message with a fixed probability.
#[derive(Clone, Debug)]
pub struct FixedLoss {
  probability: f64,
}

impl FixedLoss {
  /// Creates a [`FixedLoss`] model that drops each message with the
  /// given probability, which must be between 0.0 and 1.0 inclusive.
  #[must_use]
  pub fn new(probability: f64) -> Self {
    assert!(
      (0.0..=1.0).contains(&probability),
      "A FixedLoss probability must be between 0.0 and 1.0 inclusive"
    );
    Self { probability }
  }

  /// Returns the drop probability.
  #[must_use]
  pub fn get_probability(&self) -> f64 {
    self.probability
  }
}

impl LossModel for FixedLoss {
  fn should_drop(&mut self) -> bool {
    Task::rng().random_bool(self.probability)
  }
}

#[cfg(test)]
mod tests;
