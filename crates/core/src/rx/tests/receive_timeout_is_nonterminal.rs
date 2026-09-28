//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::ReceiveOptions;
use crate::Task;

// A receive can succeed after an earlier receive times out.
#[test]
fn test() {
  let (tx, rx) = Channel::new().pair();
  let mut executor = Executor::new();
  let receiver = executor.spawn(async move {
    assert_eq!(
      rx.receive_with_options(
        ReceiveOptions::new().timeout(Duration::from_nanos(5)),
      )
      .await,
      Err(ChannelError::TimedOut),
    );
    rx.receive().await.unwrap()
  });
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    tx.send(7).await.unwrap();
  });
  executor.run();

  assert_eq!(receiver.output(), Some(7));
}
