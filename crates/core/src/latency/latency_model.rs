//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use dyn_clone::DynClone;

/// A per-message latency model.
pub trait LatencyModel: DynClone {
  /// Returns the latency to apply to the current message.
  #[must_use]
  fn next_latency(&mut self) -> Duration;
}

dyn_clone::clone_trait_object!(LatencyModel);
