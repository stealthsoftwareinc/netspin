//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use core::time::Duration;
use std::rc::Rc;

use crate::Executor;
use crate::Moment;
use crate::Task;

// A detached task starts at the spawning task's current moment and
// runs without an output handle.
#[test]
fn test() {
  let child_moment = Rc::new(Cell::new(None));
  let mut executor = Executor::new();
  executor.spawn({
    let child_moment = child_moment.clone();
    async move {
      Task::sleep(Duration::from_secs(5)).await;
      Task::spawn_detached(async move {
        child_moment.set(Some(Task::now()));
        123
      });
    }
  });
  executor.run();

  assert_eq!(
    child_moment.get(),
    Some(Moment::ZERO + Duration::from_secs(5)),
  );
}
