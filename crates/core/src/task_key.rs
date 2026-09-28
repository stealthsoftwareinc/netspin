//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::num::NonZeroUsize;

use crate::TaskId;

/// A reusable storage slot paired with a never-reused task ID.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct TaskKey {
  pub(crate) id: TaskId,
  pub(crate) slot: NonZeroUsize,
}

impl TaskKey {
  #[must_use]
  pub(crate) fn index(self) -> usize {
    self.slot.get() - 1
  }
}
