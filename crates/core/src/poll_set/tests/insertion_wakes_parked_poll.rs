//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;
use std::rc::Rc;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::PollSet;
use crate::Task;

// Inserting a ready channel wakes a poll that's parked on an empty set.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = Rc::new(PollSet::new());
  let (tx, rx) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let receiver = executor.spawn({
    let poll_set = poll_set.clone();
    async move {
      let ready = poll_set.poll().await;
      (ready, Task::now())
    }
  });
  let inserter = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    let rx = poll_set.insert(rx);
    tx.send(123).await.unwrap();
    rx
  });
  executor.run();
  let rx = inserter.output().unwrap();
  assert_eq!(
    receiver.output().unwrap(),
    (rx.key(), Moment::from_nanos(10))
  );
}
