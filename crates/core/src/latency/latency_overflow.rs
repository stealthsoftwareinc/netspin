//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

/// The behavior of a latency block when its buffer is full.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LatencyOverflow {
  /// Waits for buffer space before admitting the new message.
  Backpressure,

  /// Drops the new message.
  #[default]
  TailDrop,
}

#[cfg(test)]
mod tests;
