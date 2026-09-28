//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;
use std::rc::Rc;

use crate::Channel;
use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Task;
use crate::test_error::TestError;

// A receiver failure discards messages that have not arrived yet.
#[test]
fn test() {
  let (tx, rx) =
    Channel::<u8, TestError>::default().local(false).pair();
  let tx = Rc::new(tx);
  let mut executor = Executor::new();
  let failure = ChannelErrorInfo::new(TestError("failed"));
  let reported_failure = failure.clone();
  let sender = tx.clone();
  let sender = executor.spawn(async move {
    sender
      .send_with_options(
        7,
        SendOptions::new().latency(Duration::from_nanos(5)),
      )
      .await
  });
  let receiver = executor.spawn(async move {
    rx.fail(reported_failure);
    (rx.receive().await, Task::now())
  });
  executor.run();
  assert_eq!(sender.output(), Some(Ok(())));
  assert_eq!(
    receiver.output(),
    Some((
      Err(ChannelError::Failed(failure.clone())),
      Moment::from_nanos(0),
    )),
  );
  assert_eq!(tx.error(), Some(ChannelError::Failed(failure)));
}
