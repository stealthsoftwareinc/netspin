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

// A message arriving at the timeout deadline wins over the timeout.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let (tx, rx) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let rx = poll_set.insert(rx);
  let key = rx.key();
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(50));
    tx.send_with_options(123, options).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    let ready =
      poll_set.poll_with_timeout(Duration::from_nanos(50)).await;
    (ready, rx.expect_receive().unwrap(), Task::now())
  });
  executor.run();
  assert_eq!(
    receiver.output().unwrap(),
    (Some(key), 123, Moment::from_nanos(50))
  );
}
