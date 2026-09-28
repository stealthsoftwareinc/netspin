//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::latency::LatencyModel;

/// A latency model that applies the same latency to every message.
#[derive(Clone, Debug)]
pub struct FixedLatency {
  latency: Duration,
}

impl FixedLatency {
  /// Creates a latency model with the given latency.
  #[must_use]
  pub fn new(latency: Duration) -> Self {
    Self { latency }
  }
}

impl LatencyModel for FixedLatency {
  fn next_latency(&mut self) -> Duration {
    self.latency
  }
}

#[cfg(test)]
mod tests;
