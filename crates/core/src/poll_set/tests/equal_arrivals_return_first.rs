//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::PollSet;
use crate::SendOptions;
use crate::Task;

// When messages arrive at the same moment, PollSet::poll() returns the
// lowest-keyed channel.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let (tx1, rx1) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let (tx2, rx2) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let rx1 = poll_set.insert(rx1);
  let _rx2 = poll_set.insert(rx2);
  let key1 = rx1.key();
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(100));
    tx1.send_with_options(1, options).await.unwrap();
    Task::sleep(Duration::from_nanos(40)).await;
    let options = SendOptions::new().latency(Duration::from_nanos(60));
    tx2.send_with_options(2, options).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    let ready = poll_set.poll().await;
    (ready, Task::now())
  });
  executor.run();
  assert_eq!(
    receiver.output().unwrap(),
    (key1, Moment::from_nanos(100))
  );
}
