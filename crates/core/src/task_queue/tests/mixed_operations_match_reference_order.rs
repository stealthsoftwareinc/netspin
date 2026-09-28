//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;
use std::collections::BTreeMap;

use super::super::TaskQueue;
use super::task;
use crate::Moment;
use crate::TaskId;
use crate::task_key::TaskKey;

// Each reference task has a scheduled event and an optional arrival
// that its initial wake must wait for. The reference scans all tasks;
// it has no slots, free list, heap, FIFO, or stale queue entries.
type Reference =
  BTreeMap<TaskId, (Option<(Moment, u64)>, Option<Moment>)>;

fn schedule(
  reference: &mut Reference,
  id: TaskId,
  moment: Moment,
  next_push: &mut u64,
) {
  reference.get_mut(&id).unwrap().0 = Some((moment, *next_push));
  *next_push += 1;
}

fn pop(
  tasks: &mut TaskQueue,
  reference: &mut Reference,
  next_push: &mut u64,
  now: &mut Moment,
) -> Option<TaskKey> {
  let expected = loop {
    let next = reference
      .iter()
      .filter_map(|(&id, &(event, _))| event.map(|event| (event, id)))
      .min();
    let Some(((moment, _), id)) = next else {
      break None;
    };
    let (event, arrival) = reference.get_mut(&id).unwrap();
    *event = None;
    if let Some(arrival) = arrival.take()
      && arrival > moment
    {
      schedule(reference, id, arrival, next_push);
      continue;
    }
    break Some((moment, id));
  };
  let actual = tasks.pop().map(|(moment, task)| {
    task.moment = moment;
    (moment, task.key)
  });
  assert_eq!(actual.map(|(moment, key)| (moment, key.id)), expected);
  actual.map(|(moment, key)| {
    *now = moment;
    key
  })
}

fn notify(
  tasks: &mut TaskQueue,
  reference: &mut Reference,
  next_push: &mut u64,
  now: Moment,
  id: TaskId,
  moment: Moment,
) {
  tasks.reschedule(tasks.key(id), moment);
  let (event, arrival) = reference.get_mut(&id).unwrap();
  if let Some(arrival) = arrival {
    *arrival = (*arrival).min(moment);
  } else if let Some((scheduled, _)) = event {
    *scheduled = (*scheduled).min(moment);
  } else {
    *arrival = Some(moment);
    schedule(reference, id, now, next_push);
  }
}

fn check(operations: impl Iterator<Item = u8>) {
  let mut tasks = TaskQueue::default();
  let mut reference = Reference::new();
  tasks.insert(Moment::ZERO, 0, task);
  reference.insert(0, (Some((Moment::ZERO, 0)), None));
  let mut next_id = 1;
  let mut next_push = 1;
  let mut now = Moment::ZERO;
  let mut retired = Vec::new();
  // Establish the current moment and an initially parked task.
  let initial =
    pop(&mut tasks, &mut reference, &mut next_push, &mut now);
  assert_eq!(initial.map(|key| key.id), Some(0));

  for (step, operation) in operations.enumerate() {
    let moment = now + Duration::from_nanos((step % 4) as u64);
    if operation == 0 || reference.is_empty() {
      tasks.insert(moment, next_id, task);
      reference.insert(next_id, (None, None));
      schedule(&mut reference, next_id, moment, &mut next_push);
      next_id += 1;
      continue;
    }
    let id = *reference.keys().nth(step % reference.len()).unwrap();
    match operation {
      1..=3 => {
        if let Some(key) =
          pop(&mut tasks, &mut reference, &mut next_push, &mut now)
        {
          if operation == 2 {
            let moment = now + Duration::from_nanos((step % 3) as u64);
            tasks.schedule(key, moment);
            schedule(&mut reference, key.id, moment, &mut next_push);
          } else if operation == 3 {
            tasks.remove(key);
            reference.remove(&key.id);
            retired.push(key);
          }
        }
      }
      4 | 7 => {
        notify(
          &mut tasks,
          &mut reference,
          &mut next_push,
          now,
          id,
          moment,
        );
        if operation == 7 {
          notify(
            &mut tasks,
            &mut reference,
            &mut next_push,
            now,
            id,
            now,
          );
        }
      }
      5 => {
        tasks.wake(tasks.key(id), now);
        if reference[&id].0.is_none() {
          schedule(&mut reference, id, now, &mut next_push);
        }
      }
      6 => {
        for &key in &retired {
          tasks.wake(key, now);
        }
      }
      _ => unreachable!(),
    }
    assert_eq!(
      tasks.slots.iter().filter(|slot| slot.is_some()).count(),
      reference.len(),
    );
  }
  // Drain scheduled tasks, then wake and drain every remaining task.
  for drain in 0..2 {
    if drain == 1 {
      let ids: Vec<_> = reference.keys().copied().collect();
      for id in ids {
        tasks.wake(tasks.key(id), now);
        schedule(&mut reference, id, now, &mut next_push);
      }
    }
    while let Some(key) =
      pop(&mut tasks, &mut reference, &mut next_push, &mut now)
    {
      tasks.remove(key);
      reference.remove(&key.id);
    }
  }
  assert!(reference.is_empty());
  assert_eq!(tasks.slots.len(), tasks.free_slots.len());
}

// Every six-operation sequence agrees with the reference, including
// parking, repeated notifications, sleep, completion, and stale wakes
// after slot reuse. Longer deterministic sequences exercise repeated
// lifecycles as well as interactions beyond the exhaustive depth.
#[test]
fn test() {
  for pattern in 0..(1_u32 << 18) {
    check((0..6).map(|step| ((pattern >> (3 * step)) & 7) as u8));
  }
  for initial in 0..128_u64 {
    let mut state = initial;
    check((0..512).map(|_| {
      state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
      (state >> 61) as u8
    }));
  }
}
