//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::Moment;
use crate::Task;

// Tx::send() uses the endpoint's configured sending timeout.
#[test]
fn test() {
  let (tx, _rx) = Channel::new().pair();
  tx.set_send_timeout(Some(Duration::from_nanos(7)));
  let mut executor = Executor::new();
  let sender = executor.spawn(async move {
    tx.send(1).await.unwrap();
    let error = tx.send(2).await.unwrap_err();
    (error, Task::now())
  });
  executor.run();

  let (error, moment) = sender.output().unwrap();
  assert_eq!(error.cause, ChannelError::TimedOut);
  assert_eq!(error.sent, 0);
  assert_eq!(error.unsent, 2);
  assert_eq!(moment, Moment::from_nanos(7));
}
