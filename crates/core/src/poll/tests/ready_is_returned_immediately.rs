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

// Calling poll() on a channel whose front message has already arrived
// returns the channel immediately, without incurring any time.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(10));
    let _ = tx.try_send_with_options(123, options).unwrap();
  });
  let receiver = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(20)).await;
    let ready = poll(&[&rx]).await;
    (ready, Task::now())
  });
  executor.run();
  assert_eq!(receiver.output().unwrap(), (0, Moment::from_nanos(20)));
}
