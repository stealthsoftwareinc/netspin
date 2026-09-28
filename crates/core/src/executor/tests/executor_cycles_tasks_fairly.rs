//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::RefCell;
use core::time::Duration;
use std::rc::Rc;

use crate::Executor;
use crate::Task;

// If multiple tasks stay at the same clock value and repeatedly await,
// the executor cycles them fairly.
#[test]
fn test() {
  let mut executor = Executor::new();
  let log = Rc::new(RefCell::new(Vec::new()));
  for id in [1, 2, 3] {
    let log = log.clone();
    executor.spawn(async move {
      for _ in 0..3 {
        log.borrow_mut().push(id);
        Task::sleep(Duration::ZERO).await;
      }
    });
  }
  executor.run();
  assert_eq!(*log.borrow(), [1, 2, 3, 1, 2, 3, 1, 2, 3]);
}
