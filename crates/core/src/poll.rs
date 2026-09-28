//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::moment::Moment;
use crate::pollable::Pollable;
use crate::task::Task;

enum ReadinessScan {
  /// The index of the first ready object.
  Ready(usize),

  /// The earliest future readiness moment.
  Pending(Moment),

  /// No object has a readiness moment.
  Empty,
}

/// Scans for the first ready object or earliest future readiness.
fn scan_readiness(
  pollables: &[impl Pollable],
  now: Moment,
) -> ReadinessScan {
  let mut earliest: Option<Moment> = None;
  for (index, pollable) in pollables.iter().enumerate() {
    let Some(ready_at) = pollable.poll_target().ready_at() else {
      continue;
    };
    if ready_at <= now {
      return ReadinessScan::Ready(index);
    }
    earliest = Some(
      earliest.map_or(ready_at, |earliest| earliest.min(ready_at)),
    );
  }
  earliest.map_or(ReadinessScan::Empty, ReadinessScan::Pending)
}

/// Waits until one of the objects is ready or the deadline is reached,
/// whichever comes first.
///
/// Returns the index of the ready object, or [`None`] if the deadline
/// is reached.
///
/// [`None`]: core::option::Option::None
pub(crate) async fn poll_until(
  pollables: &[impl Pollable],
  deadline: Option<Moment>,
) -> Option<usize> {
  let now = Task::now();
  let ready_at = match scan_readiness(pollables, now) {
    ReadinessScan::Ready(index) => return Some(index),
    ReadinessScan::Pending(ready_at) => Some(ready_at),
    ReadinessScan::Empty => None,
  };
  if deadline.is_some_and(|deadline| deadline <= now) {
    return None;
  }
  let target = match (ready_at, deadline) {
    (Some(ready_at), Some(deadline)) => Some(ready_at.min(deadline)),
    (Some(ready_at), None) => Some(ready_at),
    (None, Some(deadline)) => Some(deadline),
    (None, None) => None,
  };
  for pollable in pollables {
    pollable.poll_target().set_poll_task();
  }
  if let Some(target) = target {
    Task::sleep_until(target).await;
  } else {
    Task::park(|_| {}).await;
  }
  for pollable in pollables {
    pollable.poll_target().clear_poll_task();
  }
  let now = Task::now();
  let ready = match scan_readiness(pollables, now) {
    ReadinessScan::Ready(index) => Some(index),
    ReadinessScan::Pending(_) | ReadinessScan::Empty => None,
  };
  assert!(
    ready.is_some() || deadline.is_some_and(|deadline| deadline == now),
    "poll_until() must stop at a ready object or the deadline"
  );
  ready
}

async fn poll_helper(
  pollables: &[impl Pollable],
  timeout: Option<Duration>,
) -> Option<usize> {
  assert!(
    !pollables.is_empty(),
    "poll() must be given at least one object",
  );
  if cfg!(debug_assertions) {
    for pollable in pollables {
      pollable.poll_target().declare_poller();
    }
  }
  let deadline = timeout.map(|timeout| Task::now() + timeout);
  poll_until(pollables, deadline).await
}

/// Waits until at least one of the given [`Pollable`] objects is ready.
///
/// [`Pollable`]: crate::Pollable
#[must_use]
pub async fn poll(pollables: &[impl Pollable]) -> usize {
  poll_helper(pollables, None)
    .await
    .expect("poll_helper() without a timeout must not return None")
}

/// Waits until at least one of the given [`Pollable`] objects is ready,
/// or until the given timeout expires.
///
/// The return value is the index of the ready object in `pollables`.
/// If multiple objects are ready, the lowest index is returned.
/// If the returned object is an [`Rx`], [`Rx::expect_receive()`] is
/// guaranteed not to panic.
/// Equivalently, [`Rx::receive()`] is guaranteed to return without
/// waiting, or [`Rx::try_receive()`] is guaranteed to return `Some`.
///
/// The return value is `None` if the timeout expires.
/// If an object is ready at the same moment the timeout expires,
/// readiness wins.
///
/// Note that returning the lowest index when multiple endpoints are
/// ready does not cause starvation, as tasks are always assumed to
/// progress forward in time, not to produce an unbounded amount of
/// activity while remaining at a single moment.
///
/// If the same object is given more than once, its lowest index is
/// used.
///
/// [`Pollable`]: crate::Pollable
/// [`Rx`]: crate::Rx
/// [`Rx::expect_receive()`]: crate::Rx::expect_receive()
/// [`Rx::receive()`]: crate::Rx::receive()
/// [`Rx::try_receive()`]: crate::Rx::try_receive()
#[must_use]
pub async fn poll_with_timeout(
  pollables: &[impl Pollable],
  timeout: Duration,
) -> Option<usize> {
  poll_helper(pollables, Some(timeout)).await
}

#[cfg(test)]
mod tests;
