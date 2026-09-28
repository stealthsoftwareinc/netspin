//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::future::Future;

use crate::Spawn;
use crate::Task;
use crate::TaskOutput;

pub(crate) struct TaskSpawner;

impl Spawn for TaskSpawner {
  fn spawn<T, F>(&mut self, future: F) -> TaskOutput<T>
  where
    T: 'static,
    F: Future<Output = T> + 'static,
  {
    Task::spawn(future)
  }
}
