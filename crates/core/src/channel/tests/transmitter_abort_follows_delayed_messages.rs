//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Task;
use crate::test_error::TestError;

// A transmitter abort follows messages that have not arrived yet.
#[test]
fn test() {
  let (tx, rx) =
    Channel::<u8, TestError>::default().local(false).pair();
  let mut executor = Executor::new();
  executor.spawn(async move {
    tx.send_with_options(
      7,
      SendOptions::new().latency(Duration::from_nanos(5)),
    )
    .await
    .unwrap();
    tx.abort();
  });
  let receiver = executor.spawn(async move {
    let message = rx.receive().await;
    let message_moment = Task::now();
    let abort = rx.receive().await;
    let abort_moment = Task::now();
    (message, message_moment, abort, abort_moment)
  });
  executor.run();
  let (message, message_moment, abort, abort_moment) =
    receiver.output().unwrap();
  assert_eq!(message, Ok(7));
  assert_eq!(message_moment, Moment::from_nanos(5));
  assert!(matches!(abort, Err(ChannelError::Aborted(_))));
  assert_eq!(abort_moment, Moment::from_nanos(5));
}
