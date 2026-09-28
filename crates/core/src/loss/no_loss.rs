//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::loss::LossModel;

/// A loss model that never drops a message.
#[derive(Clone, Debug)]
pub struct NoLoss;

impl NoLoss {
  /// Creates a model that applies no loss.
  #[must_use]
  pub fn new() -> Self {
    Self
  }
}

impl Default for NoLoss {
  fn default() -> Self {
    Self::new()
  }
}

impl LossModel for NoLoss {
  fn should_drop(&mut self) -> bool {
    false
  }
}

#[cfg(test)]
mod tests;
