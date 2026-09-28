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

// Calling PollSet::poll() on channels that are all nonempty waits until
// the first message arrives.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let (tx1, rx1) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let (tx2, rx2) = Channel::<&'static str>::new()
    .capacity(None)
    .local(false)
    .pair();
  let _rx1 = poll_set.insert(rx1);
  let rx2 = poll_set.insert(rx2);
  let key2 = rx2.key();
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(100));
    tx1.send_with_options(1, options).await.unwrap();
    let options = SendOptions::new().latency(Duration::from_nanos(50));
    tx2.send_with_options("two", options).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    let ready = poll_set.poll().await;
    (ready, Task::now())
  });
  executor.run();
  assert_eq!(
    receiver.output().unwrap(),
    (key2, Moment::from_nanos(50))
  );
}
