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

// Tx::send_with_options() returns a timeout error with the unsent
// message when its configured timeout expires.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, _rx) = Channel::new().pair();
  let sender = executor.spawn(async move {
    tx.send(1).await.unwrap();
    let error = tx
      .send_with_options(
        2,
        SendOptions::new().timeout(Duration::from_nanos(50)),
      )
      .await
      .unwrap_err();
    (error, Task::now())
  });
  executor.run();
  let (error, moment) = sender.output().unwrap();
  assert_eq!(error.cause, ChannelError::TimedOut);
  assert_eq!(error.sent, 0);
  assert_eq!(error.unsent, 2);
  assert_eq!(moment, Moment::from_nanos(50));
}
