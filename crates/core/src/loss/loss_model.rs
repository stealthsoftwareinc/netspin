//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use dyn_clone::DynClone;

/// A per-message loss model.
pub trait LossModel: DynClone {
  /// Returns whether the current message should be dropped.
  #[must_use]
  fn should_drop(&mut self) -> bool;
}

dyn_clone::clone_trait_object!(LossModel);
