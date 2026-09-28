//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::Moment;
use crate::ReceiveOptions;
use crate::Task;
use crate::test_error::TestError;

// A transmitter drop interrupts a receive before its timeout.
#[test]
fn test() {
  let (tx, rx) = Channel::<u8, TestError>::default().pair();
  let mut executor = Executor::new();
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(5)).await;
    drop(tx);
  });
  let receiver = executor.spawn(async move {
    let result = rx
      .receive_with_options(
        ReceiveOptions::new().timeout(Duration::from_nanos(10)),
      )
      .await;
    (result, Task::now())
  });
  executor.run();
  assert_eq!(
    receiver.output(),
    Some((Err(ChannelError::Closed), Moment::from_nanos(5))),
  );
}
