//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use core::time::Duration;
use std::rc::Rc;

use crate::Executor;
use crate::Task;

// A staged task is added to the executor when its parent awaits, not
// only when its parent completes.
#[test]
fn test() {
  let mut executor = Executor::new();
  let child_ran = Rc::new(Cell::new(false));
  {
    let child_ran = child_ran.clone();
    executor.spawn(async move {
      let _ = Task::spawn({
        let child_ran = child_ran.clone();
        async move {
          child_ran.set(true);
        }
      });
      Task::sleep(Duration::ZERO).await;
      assert!(child_ran.get());
    });
  }
  executor.run();
}
