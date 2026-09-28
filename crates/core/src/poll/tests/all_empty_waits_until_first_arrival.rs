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

// Calling poll() on a set of channels that are all empty waits until
// the first message arrives.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx1, rx1) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let (_tx2, rx2) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(40)).await;
    let options = SendOptions::new().latency(Duration::from_nanos(10));
    tx1.send_with_options(123, options).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    let ready = poll(&[&rx1, &rx2]).await;
    (ready, Task::now())
  });
  executor.run();
  assert_eq!(receiver.output().unwrap(), (0, Moment::from_nanos(50)));
}
