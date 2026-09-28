//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::future::Future;

use crate::TaskOutput;

/// Spawns simulation tasks.
pub trait Spawn {
  /// Spawns `future`.
  fn spawn<T, F>(&mut self, future: F) -> TaskOutput<T>
  where
    T: 'static,
    F: Future<Output = T> + 'static;
}
