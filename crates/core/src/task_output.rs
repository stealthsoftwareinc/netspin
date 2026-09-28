//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use std::rc::Rc;

/// The output of a [`Task`](crate::Task).
pub struct TaskOutput<T> {
  pub(crate) output: Rc<Cell<Option<T>>>,
}

impl<T> TaskOutput<T> {
  /// Returns the output of the [`Task`](crate::Task), or `None` if the
  /// [`Task`](crate::Task) is not yet complete.
  #[must_use]
  pub fn output(self) -> Option<T> {
    self.output.take()
  }
}
