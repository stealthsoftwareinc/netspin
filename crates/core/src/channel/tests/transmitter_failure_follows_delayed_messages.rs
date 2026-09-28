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
use crate::SendOptions;
use crate::Task;
use crate::test_error::TestError;

// A transmitter failure follows messages that have not arrived yet.
#[test]
fn test() {
  let (tx, rx) =
    Channel::<u8, TestError>::default().local(false).pair();
  let mut executor = Executor::new();
  let failure = ChannelErrorInfo::new(TestError("failed"));
  let reported_failure = failure.clone();
  executor.spawn(async move {
    tx.send_with_options(
      7,
      SendOptions::new().latency(Duration::from_nanos(5)),
    )
    .await
    .unwrap();
    tx.fail(reported_failure);
  });
  let receiver = executor.spawn(async move {
    let message = rx.receive().await;
    let message_moment = Task::now();
    let failure = rx.receive().await;
    let failure_moment = Task::now();
    (message, message_moment, failure, failure_moment)
  });
  executor.run();
  assert_eq!(
    receiver.output(),
    Some((
      Ok(7),
      Moment::from_nanos(5),
      Err(ChannelError::Failed(failure)),
      Moment::from_nanos(5),
    )),
  );
}
