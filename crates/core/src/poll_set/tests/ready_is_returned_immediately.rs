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

// Calling PollSet::poll() on a channel whose front message has already
// arrived returns its key immediately, without incurring any time.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let (tx, rx) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let rx = poll_set.insert(rx);
  let key = rx.key();
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(10));
    let _ = tx.try_send_with_options(123, options).unwrap();
  });
  let receiver = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(20)).await;
    let ready = poll_set.poll().await;
    (ready, rx.expect_receive().unwrap(), Task::now())
  });
  executor.run();
  assert_eq!(
    receiver.output().unwrap(),
    (key, 123, Moment::from_nanos(20))
  );
}
