//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::RefCell;
use core::task::Waker;
use core::time::Duration;
use std::rc::Rc;

use crate::Executor;
use crate::Moment;
use crate::Task;

// A task woken by another task resumes at the waking task's moment.
#[test]
fn test() {
  let waker = Rc::new(RefCell::new(None::<Waker>));
  let mut executor = Executor::new();
  let output = executor.spawn({
    let waker = waker.clone();
    async move {
      Task::park(|new_waker| {
        *waker.borrow_mut() = Some(new_waker.clone());
      })
      .await;
      Task::now()
    }
  });
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(100)).await;
    waker.borrow_mut().take().unwrap().wake();
  });
  executor.run();
  assert_eq!(output.output(), Some(Moment::from_nanos(100)));
}
