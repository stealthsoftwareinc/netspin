//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::PollSet;
use crate::Task;

// PollSet::poll() cleans up its registrations before being called again.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let (tx, rx) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let rx = poll_set.insert(rx);
  let key = rx.key();
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    tx.send(1).await.unwrap();
    Task::sleep(Duration::from_nanos(10)).await;
    tx.send(2).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    let ready1 = poll_set.poll().await;
    let message1 = rx.expect_receive().unwrap();
    let ready2 = poll_set.poll().await;
    let message2 = rx.expect_receive().unwrap();
    (ready1, message1, ready2, message2, Task::now())
  });
  executor.run();
  assert_eq!(
    receiver.output().unwrap(),
    (key, 1, key, 2, Moment::from_nanos(20))
  );
}
