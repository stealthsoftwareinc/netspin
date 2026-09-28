//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::RefCell;
use core::time::Duration;
use std::rc::Rc;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Task;

#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let result = Rc::new(RefCell::new(None));
  let out = result.clone();
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(456));
    tx.send_with_options(123, options).await.unwrap();
  });
  executor.spawn(async move {
    let message = rx.receive().await.unwrap();
    *out.borrow_mut() = Some((message, Task::now()));
  });
  executor.run();
  assert_eq!(*result.borrow(), Some((123, Moment::from_nanos(456))));
}
