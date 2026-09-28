//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::convert::Infallible;

use rand::Rng;
use rand::TryRng;

use crate::executor::TASK_RNG;
use crate::task::Task;

/// A forwarder to the running [`Executor`]'s RNG.
///
/// Each [`Rng`] method only borrows the RNG for the duration of the
/// underlying call, so a `TaskRng` can be held across await points.
///
/// [`Executor`]: crate::Executor
/// [`Rng`]: rand::Rng
pub struct TaskRng;

fn with_rng<R>(f: impl FnOnce(&mut dyn Rng) -> R) -> R {
  TASK_RNG.with_borrow_mut(|rng| {
    f(&mut **rng
      .as_mut()
      .expect("Task::rng() must only be used within a task"))
  })
}

impl TryRng for TaskRng {
  type Error = Infallible;

  fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
    Task::assert_running("TaskRng::try_next_u32()");
    Ok(with_rng(|rng| rng.next_u32()))
  }

  fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
    Task::assert_running("TaskRng::try_next_u64()");
    Ok(with_rng(|rng| rng.next_u64()))
  }

  fn try_fill_bytes(
    &mut self,
    dst: &mut [u8],
  ) -> Result<(), Self::Error> {
    Task::assert_running("TaskRng::try_fill_bytes()");
    with_rng(|rng| rng.fill_bytes(dst));
    Ok(())
  }
}

#[cfg(test)]
mod tests;
