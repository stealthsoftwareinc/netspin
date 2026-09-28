//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::SendOptions;
use crate::Task;

// A timed-out send can be retried on the same channel.
#[test]
fn test() {
  let (tx, rx) = Channel::new().pair();
  let mut executor = Executor::new();
  let sender = executor.spawn(async move {
    tx.send(1).await.unwrap();
    let error = tx
      .send_with_options(
        2,
        SendOptions::new().timeout(Duration::from_nanos(5)),
      )
      .await
      .unwrap_err();
    assert_eq!(error.cause, ChannelError::TimedOut);
    assert_eq!(error.sent, 0);
    assert_eq!(tx.error(), None);
    tx.send(error.unsent).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    [rx.receive().await.unwrap(), rx.receive().await.unwrap()]
  });
  executor.run();

  assert_eq!(sender.output(), Some(()));
  assert_eq!(receiver.output(), Some([1, 2]));
}
