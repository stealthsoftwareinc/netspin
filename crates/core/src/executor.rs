//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use core::cell::RefCell;
use core::future::Future;
use core::task::Context;
use core::task::Poll;
use core::task::Waker;
use core::time::Duration;
use std::rc::Rc;
use std::sync::Arc;
use std::task::Wake;

use rand::Rng;
use rand::SeedableRng as _;
use tokio::runtime::Handle;

use crate::block_on;
use crate::fast_rng::FastRng;
use crate::moment::Moment;
use crate::spawn::Spawn;
use crate::task::Task;
use crate::task::TaskFuture;
use crate::task::TaskId;
use crate::task_key::TaskKey;
use crate::task_output::TaskOutput;
use crate::task_queue::TaskQueue;

const ASYNC_RULE_VIOLATION: &str =
  "netspin_core::Executor::run(): Async rule violation";

/// Indicates how a [`Task`] became suspended.
///
/// [`Task`]: crate::Task
pub(crate) enum Suspend {
  /// Indicates the [`Task`] awaited a [`Task::sleep_until()`] call.
  ///
  /// [`Task`]: crate::Task
  /// [`Task::sleep_until()`]: crate::Task::sleep_until()
  Pause(Moment),

  /// Indicates the [`Task`] awaited a [`Task::park()`] call.
  ///
  /// [`Task`]: crate::Task
  /// [`Task::park()`]: crate::Task::park()
  Park,

  /// Indicates the [`Task`] awaited a [`Task::work()`] call.
  ///
  /// [`Task`]: crate::Task
  /// [`Task::work()`]: crate::Task::work()
  Work(Moment),
}

thread_local! {
  /// Indicates whether this thread is inside Executor::run().
  static RUNNING: Cell<bool> = const { Cell::new(false) };

  /// The scheduling handle of the current task.
  pub(crate) static TASK_KEY: Cell<Option<TaskKey>> = const { Cell::new(None) };

  /// The clock value of the current task.
  pub(crate) static TASK_NOW: Cell<Option<Moment>> = const { Cell::new(None) };

  /// Indicates how the current task became suspended.
  pub(crate) static SUSPEND: Cell<Option<Suspend>> = const { Cell::new(None) };

  /// The running executor's random number generator.
  pub(crate) static TASK_RNG: RefCell<Option<Box<dyn Rng>>> =
    const { RefCell::new(None) };

  /// Reschedules staged by the current poll() call.
  static RESCHEDULES: RefCell<Vec<(TaskKey, Moment)>> =
    const { RefCell::new(Vec::new()) };

  /// Tasks spawned by the current poll() call.
  pub(crate) static SPAWNS: RefCell<Vec<TaskFuture>> =
    const { RefCell::new(Vec::new()) };

  /// Tasks flagged to be woken and their wake moments.
  static WOKEN: RefCell<Vec<(TaskKey, Moment)>> =
    const { RefCell::new(Vec::new()) };
}

struct RunningGuard;

impl Drop for RunningGuard {
  fn drop(&mut self) {
    RUNNING.set(false);
    SUSPEND.set(None);
    TASK_KEY.set(None);
    TASK_NOW.set(None);
    TASK_RNG.with_borrow_mut(|rng| *rng = None);
    block_on::set_handle(None);
    RESCHEDULES.with_borrow_mut(|reschedules| reschedules.clear());
    SPAWNS.with_borrow_mut(|spawns| spawns.clear());
    WOKEN.with_borrow_mut(|woken| woken.clear());
  }
}

/// Stages a request to wake a polling task at the given moment or
/// bring its existing wakeup forward.
pub(crate) fn reschedule(key: TaskKey, moment: Moment) {
  RESCHEDULES
    .with_borrow_mut(|reschedules| reschedules.push((key, moment)));
}

/// A waker that unparks a task.
///
/// Only wake calls made within a running task schedule the woken task.
/// Calls made outside a running task are ignored.
/// Such calls can occur when an executor drops its remaining parked
/// tasks during teardown.
struct TaskWaker {
  key: TaskKey,
}

impl Wake for TaskWaker {
  fn wake(self: Arc<Self>) {
    let Some(moment) = TASK_NOW.get() else {
      return;
    };
    WOKEN.with_borrow_mut(|woken| woken.push((self.key, moment)));
  }
}

/// A single-threaded executor for discrete-event simulation.
///
/// Each task keeps track of what time it is independently from other
/// tasks by keeping its own clock value.
/// The clock unit is arbitrary, but nanoseconds is the typical choice.
///
/// Tasks spawned directly on an executor start at [`Moment::ZERO`].
/// Tasks spawned by a running task start at that task's current
/// [`Moment`].
///
/// A task can use the `Task::now()` function to retrieve its current
/// clock value, and the `Task::sleep()` and `Task::sleep_until()`
/// functions to advance its clock value.
///
/// Tasks are run in furthest-behind order.
/// In other words, any time a task is running, its current clock value
/// (returned by `Task::now()`) is guaranteed to be less than or equal
/// to the clock value of all other tasks.
///
/// [`Moment`]: crate::Moment
/// [`Moment::ZERO`]: crate::Moment::ZERO
pub struct Executor {
  /// The maximum amount of time the simulation will run for.
  horizon: Option<Duration>,

  tasks: TaskQueue,

  /// The ID to assign to the next spawned task.
  next_id: TaskId,

  /// The random number generator.
  rng: Option<Box<dyn Rng>>,

  /// A handle to the runtime used by [`Task::block_on()`].
  ///
  /// [`Task::block_on()`]: crate::Task::block_on()
  runtime: Handle,
}

impl Executor {
  /// Creates an executor with the default settings.
  #[must_use]
  pub fn new() -> Self {
    Self {
      tasks: TaskQueue::default(),
      horizon: None,
      next_id: 0,
      rng: Some(Box::new(FastRng::from_rng(&mut rand::rng()))),
      runtime: block_on::handle(),
    }
  }

  /// Sets the maximum amount of time the simulation will run for.
  ///
  /// The horizon is a [`Duration`] since [`Moment::ZERO`].
  /// Tasks are abandoned once they reach the horizon.
  ///
  /// The default is no horizon.
  ///
  /// [`Duration`]: core::time::Duration
  /// [`Moment::ZERO`]: crate::Moment::ZERO
  #[must_use]
  pub fn horizon(mut self, horizon: Duration) -> Self {
    self.horizon = Some(horizon);
    self
  }

  /// Sets the random number generator.
  ///
  /// The default is a [`FastRng`] seeded by [`rand::rng()`].
  ///
  /// [`FastRng`]: crate::FastRng
  /// [`rand::rng()`]: rand::rng()
  #[must_use]
  pub fn rng(mut self, rng: impl Rng + 'static) -> Self {
    self.rng = Some(Box::new(rng));
    self
  }

  /// Runs all spawned tasks.
  ///
  /// Running continues until one of the following is true for every
  /// task:
  ///
  /// 1. The task returned.
  /// 2. The task is [parked].
  /// 3. The task reached the [horizon].
  ///
  /// [horizon]: Executor::horizon
  /// [parked]: crate
  pub fn run(mut self) {
    let running = RUNNING.replace(true);
    assert!(!running, "Executor::run() must not be nested");
    let _guard = RunningGuard;

    TASK_RNG.with_borrow_mut(|rng| *rng = self.rng.take());
    block_on::set_handle(Some(self.runtime.clone()));

    while let Some((key, task)) = self.tasks.pop() {
      if self
        .horizon
        .is_some_and(|horizon| key.to_duration() >= horizon)
      {
        // This case means the furthest-behind unparked task has reached
        // the horizon, which means all unparked tasks have reached the
        // horizon, which means we're done. It's also impossible that
        // we've run any tasks beyond the horizon, as we always run
        // furthest-behind first.
        break;
      }

      task.moment = key;
      TASK_KEY.set(Some(task.key));
      TASK_NOW.set(Some(task.moment));

      let mut cx = Context::from_waker(&task.waker);

      let previous_suspend = SUSPEND.take();
      assert!(
        previous_suspend.is_none(),
        "{ASYNC_RULE_VIOLATION}: A task suspension was already \
         registered at the start of a poll.",
      );

      let poll = task.future.as_mut().poll(&mut cx);
      let suspend = SUSPEND.take();
      let task_key = task.key;
      let moment = task.moment;

      // Add any tasks spawned by the just-polled task.
      SPAWNS.with_borrow_mut(|spawns| {
        for future in spawns.drain(..) {
          self.spawn_task(moment, future);
        }
      });

      match (poll, suspend) {
        (Poll::Ready(()), None) => self.tasks.remove(task_key),
        (Poll::Ready(()), Some(_)) => {
          panic!(
            "{ASYNC_RULE_VIOLATION}: A task returned ready after \
             registering a suspension.",
          );
        }
        (Poll::Pending, Some(Suspend::Pause(new_clock))) => {
          assert!(
            new_clock >= moment,
            "Tasks must not move backward in time"
          );
          self.tasks.schedule(task_key, new_clock);
        }
        (Poll::Pending, Some(Suspend::Park)) => {}
        (Poll::Pending, Some(Suspend::Work(new_clock))) => {
          assert!(
            new_clock >= moment,
            "netspin_core::Executor::run(): Tasks must not move backward \
             in time.",
          );
          self.tasks.schedule(task_key, new_clock);
        }
        (Poll::Pending, None) => {
          panic!(
            "{ASYNC_RULE_VIOLATION}: A task returned pending without \
             registering a suspension.",
          );
        }
      }

      // Apply any reschedules staged by the poll() call.
      RESCHEDULES.with_borrow_mut(|reschedules| {
        for (id, key) in reschedules.drain(..) {
          self.tasks.reschedule(id, key);
        }
      });

      // Unpark any tasks flagged by the poll() call.
      WOKEN.with_borrow_mut(|woken| {
        for (id, moment) in woken.drain(..) {
          self.tasks.wake(id, moment);
        }
      });
    }
  }

  pub fn spawn<T, F>(&mut self, future: F) -> TaskOutput<T>
  where
    T: 'static,
    F: Future<Output = T> + 'static,
  {
    let output = Rc::new(Cell::new(None));
    self.spawn_task(
      Moment::ZERO,
      Box::pin({
        let output = output.clone();
        async move {
          output.set(Some(future.await));
        }
      }),
    );
    TaskOutput { output }
  }

  fn spawn_task(&mut self, moment: Moment, future: TaskFuture) {
    let id = self.next_id;
    self.tasks.insert(moment, id, |key| Task {
      future,
      key,
      moment,
      waker: Waker::from(Arc::new(TaskWaker { key })),
    });
    self.next_id =
      self.next_id.checked_add(1).expect("TaskId overflow");
  }
}

impl Default for Executor {
  fn default() -> Self {
    Self::new()
  }
}

impl Spawn for Executor {
  fn spawn<T, F>(&mut self, future: F) -> TaskOutput<T>
  where
    T: 'static,
    F: Future<Output = T> + 'static,
  {
    Executor::spawn(self, future)
  }
}

#[cfg(test)]
mod tests;
