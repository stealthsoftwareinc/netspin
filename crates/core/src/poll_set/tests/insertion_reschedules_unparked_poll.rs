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
use crate::SendOptions;
use crate::Task;

// Inserting a channel reschedules a poll that's unparked for a later
// arrival.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = Rc::new(PollSet::new());
  let (tx1, rx1) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let (tx2, rx2) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let _rx1 = poll_set.insert(rx1);
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(100));
    tx1.send_with_options(1, options).await.unwrap();
  });
  let receiver = executor.spawn({
    let poll_set = poll_set.clone();
    async move {
      let ready = poll_set.poll().await;
      (ready, Task::now())
    }
  });
  let inserter = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(20)).await;
    let rx2 = poll_set.insert(rx2);
    let options = SendOptions::new().latency(Duration::from_nanos(10));
    tx2.send_with_options(2, options).await.unwrap();
    rx2
  });
  executor.run();
  let rx2 = inserter.output().unwrap();
  assert_eq!(
    receiver.output().unwrap(),
    (rx2.key(), Moment::from_nanos(30))
  );
}
