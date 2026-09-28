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

// Removing a channel wakes a parked poll so it can rebuild its channel
// registrations.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = Rc::new(PollSet::new());
  let (_tx1, rx1) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let (tx2, rx2) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let rx1 = poll_set.insert(rx1);
  let receiver = executor.spawn({
    let poll_set = poll_set.clone();
    async move {
      let ready = poll_set.poll().await;
      (ready, Task::now())
    }
  });
  let mutator = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    drop(rx1);
    Task::sleep(Duration::from_nanos(10)).await;
    let rx2 = poll_set.insert(rx2);
    tx2.send(2).await.unwrap();
    rx2
  });
  executor.run();
  let rx2 = mutator.output().unwrap();
  assert_eq!(
    receiver.output().unwrap(),
    (rx2.key(), Moment::from_nanos(20))
  );
}
