//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use core::task::Waker;
use core::time::Duration;
use std::rc::Rc;

use crate::executor::SPAWNS;
use crate::executor::SUSPEND;
use crate::executor::Suspend;
use crate::executor::TASK_KEY;
use crate::executor::TASK_NOW;
use crate::moment::Moment;
use crate::spawn::Spawn;
use crate::task_key::TaskKey;
use crate::task_output::TaskOutput;
use crate::task_rng::TaskRng;
use crate::task_spawner::TaskSpawner;
use crate::task_work::TaskWork;

const ASYNC_RULE_VIOLATION: &str =
  "netspin_core::Task::suspend(): Async rule violation";

/// A unique task ID.
pub type TaskId = u64;

pub(crate) type TaskFuture = Pin<Box<dyn Future<Output = ()>>>;

/// An asynchronous thread of execution in the simulation.
///
/// This type provides executor-related functions that can be called by
/// tasks.
/// For example, [`Task::now()`] returns the current [`Moment`] of the
/// calling task, and [`Task::spawn()`] allows a task to spawn other
/// tasks.
///
/// [`Moment`]: crate::Moment
/// [`Task::now()`]: crate::Task::now()
/// [`Task::spawn()`]: crate::Task::spawn()
pub struct Task {
  pub(crate) future: TaskFuture,

  pub(crate) key: TaskKey,

  /// The task's current [`Moment`].
  ///
  /// This begins as the moment at which the task was spawned.
  /// Immediately before each call to [`Future::poll()`], the executor
  /// updates it to the moment at which that poll occurs.
  /// An initial wake may also advance it without polling the future.
  ///
  /// When the task is scheduled in the [`TaskQueue`] collection,
  /// its scheduling key records the moment at which it will be polled.
  /// The scheduled moment is always greater than or equal to this
  /// moment.
  ///
  /// [`Future::poll()`]: core::future::Future::poll()
  /// [`Moment`]: crate::Moment
  /// [`TaskQueue`]: crate::task_queue::TaskQueue
  pub(crate) moment: Moment,

  pub(crate) waker: Waker,
}

impl Task {
  pub(crate) fn assert_running(f: &str) {
    if !Task::is_running() {
      panic!("{f} must only be called within a task");
    }
  }

  /// Runs a normal [`Future`] to completion.
  ///
  /// The future will be run to completion using an auxiliary Tokio
  /// [`Runtime`], blocking as if it had been executed synchronously
  /// (note that this function is not `async`).
  /// This allows self-contained work to be performed with third-party
  /// libraries that require `async`.
  ///
  /// This function must only be called from a running task.
  ///
  /// [`Future`]: core::future::Future
  /// [`Runtime`]: tokio::runtime::Runtime
  pub fn block_on<F>(future: F) -> F::Output
  where
    F: Future,
  {
    Task::assert_running("Task::block_on()");
    crate::block_on::block_on(future)
  }

  #[must_use]
  pub fn id() -> TaskId {
    TASK_KEY.get().map(|key| key.id).unwrap_or_else(|| {
      panic!("Task::id() must only be called within a task")
    })
  }

  #[must_use]
  pub(crate) fn key() -> TaskKey {
    TASK_KEY.get().expect(
      "netspin_core::Task::key(): Must be called within a task.",
    )
  }

  #[must_use]
  pub(crate) fn is_running() -> bool {
    TASK_KEY.get().is_some()
  }

  #[must_use]
  pub fn now() -> Moment {
    TASK_NOW.get().unwrap_or_else(|| {
      panic!("Task::now() must only be called within a task")
    })
  }

  /// Parks the calling task, making it ineligible for furthest-behind
  /// execution, until it is woken.
  pub(crate) fn park(
    register: impl FnOnce(&Waker) + Unpin,
  ) -> impl Future<Output = ()> {
    Park {
      ready: false,
      register: Some(register),
    }
  }

  /// Returns a forwarder to the running [`Executor`]'s RNG.
  ///
  /// [`Executor`]: crate::Executor
  #[must_use]
  pub fn rng() -> TaskRng {
    Task::assert_running("Task::rng()");
    TaskRng
  }

  /// Suspends the calling task for the given duration.
  pub fn sleep(duration: Duration) -> impl Future<Output = ()> {
    Task::assert_running("Task::sleep()");
    Task::sleep_until(Task::now() + duration)
  }

  /// Suspends the calling task until the given moment.
  pub fn sleep_until(moment: Moment) -> impl Future<Output = ()> {
    Task::assert_running("Task::sleep_until()");
    Pause {
      ready: false,
      clock: moment,
    }
  }

  /// Spawns a new task at the calling task's current [`Moment`].
  ///
  /// If you don't need the task's output, consider using
  /// [`spawn_detached()`] instead, which has better performance.
  ///
  /// This function must only be called from a running task.
  ///
  /// [`Moment`]: crate::Moment
  /// [`spawn_detached()`]: crate::Task::spawn_detached
  #[must_use = "use Task::spawn_detached() if the task's output is not needed"]
  pub fn spawn<T, F>(future: F) -> TaskOutput<T>
  where
    T: 'static,
    F: Future<Output = T> + 'static,
  {
    Task::assert_running("Task::spawn()");
    let output = Rc::new(Cell::new(None));
    SPAWNS.with_borrow_mut(|spawns| {
      spawns.push(Box::pin({
        let output = output.clone();
        async move {
          output.set(Some(future.await));
        }
      }));
    });
    TaskOutput { output }
  }

  /// Spawns a new task at the calling task's current [`Moment`] and
  /// ignores its output.
  ///
  /// This function is the same as [`spawn()`], except it does not
  /// return the task's output.
  /// This provides better performance, especially when spawning large
  /// numbers of tasks.
  ///
  /// This function must only be called from a running task.
  ///
  /// [`Moment`]: crate::Moment
  /// [`spawn()`]: crate::Task::spawn
  pub fn spawn_detached<F>(future: F)
  where
    F: Future + 'static,
  {
    Task::assert_running("Task::spawn_detached()");
    SPAWNS.with_borrow_mut(|spawns| {
      spawns.push(Box::pin(async move {
        let _ = future.await;
      }));
    });
  }

  /// Returns a spawner for the running executor.
  ///
  /// This function must only be called from a running task.
  #[must_use]
  pub fn spawner() -> impl Spawn {
    Task::assert_running("Task::spawner()");
    TaskSpawner
  }

  /// Suspends the current task for the given reason.
  pub(crate) fn suspend(suspend: Suspend) -> Poll<()> {
    let previous = SUSPEND.replace(Some(suspend));
    assert!(
      previous.is_none(),
      "{ASYNC_RULE_VIOLATION}: A task registered more than one \
       suspension during a single poll.",
    );
    Poll::Pending
  }

  pub(crate) fn try_now() -> Option<Moment> {
    TASK_NOW.get()
  }

  /// Performs modeled active work for the given duration.
  ///
  /// This advances the calling task's clock and allows other tasks to
  /// run during the interval, like [`sleep()`].
  /// Unlike sleeping, this operation identifies time spent actively
  /// servicing work so execution-resource models can distinguish it.
  ///
  /// [`sleep()`]: Task::sleep()
  pub fn work(duration: Duration) -> impl Future<Output = ()> {
    Task::assert_running("Task::work()");
    TaskWork::new(Task::now() + duration)
  }
}

#[cfg(test)]
mod tests;

struct Pause {
  ready: bool,
  clock: Moment,
}

impl Future for Pause {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
    if self.ready {
      return Poll::Ready(());
    }
    self.ready = true;
    Task::suspend(Suspend::Pause(self.clock))
  }
}

pub(crate) struct Park<F> {
  ready: bool,
  register: Option<F>,
}

impl<F: FnOnce(&Waker) + Unpin> Future for Park<F> {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
    if self.ready {
      return Poll::Ready(());
    }
    self.ready = true;
    self.register.take().unwrap()(cx.waker());
    Task::suspend(Suspend::Park)
  }
}
