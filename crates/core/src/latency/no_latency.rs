//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::latency::LatencyModel;

/// A latency model that applies no latency.
#[derive(Clone, Debug)]
pub struct NoLatency;

impl NoLatency {
  /// Creates a model that applies no latency.
  #[must_use]
  pub fn new() -> Self {
    Self
  }
}

impl Default for NoLatency {
  fn default() -> Self {
    Self::new()
  }
}

impl LatencyModel for NoLatency {
  fn next_latency(&mut self) -> Duration {
    Duration::ZERO
  }
}

#[cfg(test)]
mod tests;
