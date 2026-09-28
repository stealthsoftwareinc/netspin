//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Executor;
use crate::PollSet;
use crate::Signal;

// Reenabled entries retain their original priority among ready entries,
// including entries inserted while they were disabled.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let mut entries = Vec::new();
  for _ in 0..3 {
    let (trigger, signal) = Signal::pair();
    trigger.trigger();
    entries.push(poll_set.insert(signal));
  }
  entries[0].disable();
  entries[1].disable();
  let (trigger, signal) = Signal::pair();
  trigger.trigger();
  entries.push(poll_set.insert(signal));
  entries[1].enable();
  entries[0].enable();

  let waiter = executor.spawn(async move {
    for entry in entries {
      assert_eq!(poll_set.poll().await, entry.key());
      entry.disable();
    }
  });
  executor.run();
  assert!(waiter.output().is_some());
}
