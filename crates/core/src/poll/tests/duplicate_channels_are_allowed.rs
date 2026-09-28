//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Task;
use crate::poll;

// The same channel can occur more than once in a poll() call. The first
// occurrence is returned when the channel becomes ready.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(40)).await;
    let options = SendOptions::new().latency(Duration::from_nanos(10));
    tx.send_with_options(123, options).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    let ready = poll(&[&rx, &rx]).await;
    (ready, rx.expect_receive().unwrap(), Task::now())
  });
  executor.run();
  assert_eq!(
    receiver.output().unwrap(),
    (0, 123, Moment::from_nanos(50))
  );
}
