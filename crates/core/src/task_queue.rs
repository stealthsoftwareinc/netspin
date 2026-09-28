//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cmp::Reverse;
use core::num::NonZeroUsize;
use std::collections::BinaryHeap;
use std::collections::VecDeque;

use crate::moment::Moment;
use crate::task::Task;
use crate::task::TaskId;
use crate::task_key::TaskKey;

type PushId = u64;
type ScheduleKey = Reverse<(Moment, PushId, usize)>;

const QUEUE_REBUILD_RATIO: usize = 2;
const QUEUE_REBUILD_SLACK: usize = 1024;

struct ScheduledTask {
  task: Task,
  schedule: Option<(Moment, PushId)>,
  ready_at: Option<Moment>,
  in_heap: bool,
}

/// Owns tasks in reusable slots, scheduling through direct handles.
///
/// Tasks resume in moment order, with ties broken by push order.
/// Current-moment tasks use a FIFO, while future and rescheduled tasks
/// use a priority queue.
/// Parked tasks have no scheduled moment.
/// Completed tasks release their slots for reuse, so slot storage is
/// bounded by the peak number of live tasks rather than total spawns.
/// Handles include the task ID so stale wakes cannot affect a new task
/// that reuses the same slot.
/// Queue entries use never-reused push IDs to reject stale entries.
#[derive(Default)]
pub(crate) struct TaskQueue {
  current_moment: Option<Moment>,
  current_tasks: VecDeque<(PushId, usize)>,
  heap_tasks: usize,
  next_push_id: PushId,
  queue: BinaryHeap<ScheduleKey>,
  slots: Vec<Option<ScheduledTask>>,
  free_slots: Vec<usize>,
}

impl TaskQueue {
  fn pop_index(&mut self) -> Option<(Moment, usize)> {
    loop {
      if let Some(&(push_id, index)) = self.current_tasks.front() {
        let moment = self.current_moment.unwrap();
        if self
          .queue
          .peek()
          .is_none_or(|queued| (moment, push_id, index) <= queued.0)
        {
          self.current_tasks.pop_front();
          return Some((moment, index));
        }
      }
      let Reverse((moment, push_id, index)) = self.queue.pop()?;
      if self
        .slots
        .get(index)
        .and_then(Option::as_ref)
        .is_none_or(|task| task.schedule != Some((moment, push_id)))
      {
        continue;
      }
      debug_assert!(
        self.current_moment.is_none_or(|current| current <= moment),
        "netspin_core::task_queue::TaskQueue::pop_index(): Task was \
         scheduled before the current moment.",
      );
      self.current_moment = Some(moment);
      self.heap_tasks -= 1;
      return Some((moment, index));
    }
  }

  pub(crate) fn pop(&mut self) -> Option<(Moment, &mut Task)> {
    let (moment, index) = loop {
      let (moment, index) = self.pop_index()?;
      let entry = self.slots[index].as_mut().unwrap();
      entry.schedule = None;
      entry.in_heap = false;
      let Some(ready_at) = entry.ready_at.take() else {
        break (moment, index);
      };
      if ready_at <= moment {
        break (moment, index);
      }
      // Keep the initial wake's place in current-moment order, then
      // schedule its real poll without polling an empty future.
      entry.task.moment = moment;
      let key = entry.task.key;
      self.schedule(key, ready_at);
    };
    let entry = self.slots[index].as_mut().unwrap();
    Some((moment, &mut entry.task))
  }

  pub(crate) fn insert(
    &mut self,
    moment: Moment,
    id: TaskId,
    make_task: impl FnOnce(TaskKey) -> Task,
  ) {
    let index = self.free_slots.pop().unwrap_or_else(|| {
      self.slots.push(None);
      self.slots.len() - 1
    });
    let key = TaskKey {
      id,
      slot: NonZeroUsize::new(index + 1).unwrap(),
    };
    let entry = ScheduledTask {
      task: make_task(key),
      schedule: None,
      ready_at: None,
      in_heap: false,
    };
    debug_assert_eq!(
      entry.task.key, key,
      "netspin_core::task_queue::TaskQueue::insert(): Task handle \
       mismatch.",
    );
    self.slots[index] = Some(entry);
    self.schedule(key, moment);
  }

  pub(crate) fn schedule(&mut self, key: TaskKey, moment: Moment) {
    let entry = self.slots[key.index()].as_mut().unwrap();
    debug_assert_eq!(
      entry.task.key, key,
      "netspin_core::task_queue::TaskQueue::schedule(): Task handle \
       mismatch.",
    );
    debug_assert!(
      entry.schedule.is_none(),
      "netspin_core::task_queue::TaskQueue::schedule(): Task is \
       already scheduled.",
    );
    debug_assert!(
      self.current_moment.is_none_or(|current| current <= moment),
      "netspin_core::task_queue::TaskQueue::schedule(): Task was \
       scheduled before the current moment.",
    );
    let push_id = self.next_push_id;
    self.next_push_id = push_id.checked_add(1).expect(
      "netspin_core::task_queue::TaskQueue::schedule(): Push ID \
       overflow.",
    );
    entry.schedule = Some((moment, push_id));
    entry.in_heap = self.current_moment != Some(moment);
    if entry.in_heap {
      self.queue.push(Reverse((moment, push_id, key.index())));
      self.heap_tasks += 1;
    } else {
      self.current_tasks.push_back((push_id, key.index()));
    }
  }

  pub(crate) fn remove(&mut self, key: TaskKey) {
    let entry = self.slots[key.index()].take().unwrap();
    debug_assert_eq!(
      entry.task.key, key,
      "netspin_core::task_queue::TaskQueue::remove(): Task handle \
       mismatch.",
    );
    debug_assert!(
      entry.schedule.is_none(),
      "netspin_core::task_queue::TaskQueue::remove(): Task is \
       still scheduled.",
    );
    self.free_slots.push(key.index());
  }

  pub(crate) fn wake(&mut self, key: TaskKey, moment: Moment) {
    let Some(entry) = self
      .slots
      .get_mut(key.index())
      .and_then(Option::as_mut)
      .filter(|entry| entry.task.key == key)
    else {
      return;
    };
    if entry.schedule.is_some() {
      return;
    }
    let moment = moment.max(entry.task.moment);
    self.schedule(key, moment);
  }

  pub(crate) fn reschedule(&mut self, key: TaskKey, moment: Moment) {
    debug_assert!(
      self.current_moment.is_none_or(|current| current <= moment),
      "netspin_core::task_queue::TaskQueue::reschedule(): Task was \
       scheduled before the current moment.",
    );
    let entry = self.slots[key.index()].as_mut().unwrap();
    assert_eq!(
      entry.task.key, key,
      "netspin_core::task_queue::TaskQueue::reschedule(): Task handle \
       mismatch.",
    );
    if let Some(ready_at) = &mut entry.ready_at {
      *ready_at = (*ready_at).min(moment);
      return;
    }
    let Some((scheduled, push_id)) = entry.schedule else {
      entry.ready_at = Some(moment.max(entry.task.moment));
      self.schedule(key, self.current_moment.unwrap());
      return;
    };
    if scheduled <= moment {
      return;
    }
    assert!(
      entry.task.moment <= moment,
      "netspin_core::task_queue::TaskQueue::reschedule(): Task was \
       attempted to be rescheduled back in time.",
    );
    entry.schedule = Some((moment, push_id));
    self.queue.push(Reverse((moment, push_id, key.index())));
    self.rebuild_queue_if_needed();
  }

  fn rebuild_queue_if_needed(&mut self) {
    let maximum_len = self
      .heap_tasks
      .saturating_mul(QUEUE_REBUILD_RATIO)
      .saturating_add(QUEUE_REBUILD_SLACK);
    if self.queue.len() <= maximum_len {
      return;
    }
    self.queue = self
      .slots
      .iter()
      .filter_map(Option::as_ref)
      .filter_map(|entry| {
        let (moment, push_id) = entry.schedule?;
        entry.in_heap.then_some(Reverse((
          moment,
          push_id,
          entry.task.key.index(),
        )))
      })
      .collect();
  }
}

#[cfg(test)]
impl TaskQueue {
  fn key(&self, id: TaskId) -> TaskKey {
    self
      .slots
      .iter()
      .filter_map(Option::as_ref)
      .find(|entry| entry.task.key.id == id)
      .unwrap()
      .task
      .key
  }

  fn queue_len(&self) -> usize {
    self.queue.len() + self.current_tasks.len()
  }
}

#[cfg(test)]
mod tests;
