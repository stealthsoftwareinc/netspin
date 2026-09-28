//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::Moment;
use crate::Task;
use crate::poll;
use crate::test_error::TestError;

// A failure makes a receiver ready.
#[test]
fn test() {
  let (tx, rx) = Channel::<u8>::new().fallible::<TestError>().pair();
  let mut executor = Executor::new();
  let failure = ChannelErrorInfo::new(TestError("failed"));
  let reported_failure = failure.clone();
  let receiver = executor.spawn(async move {
    let ready = poll(&[&rx]).await;
    (ready, rx.receive().await, Task::now())
  });
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(1)).await;
    tx.fail(reported_failure);
  });
  executor.run();

  assert_eq!(
    receiver.output(),
    Some((
      0,
      Err(ChannelError::Failed(failure)),
      Moment::ZERO + Duration::from_nanos(1),
    )),
  );
}
