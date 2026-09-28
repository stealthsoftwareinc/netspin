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

// Send readiness wins when space becomes available at the timeout.
#[test]
fn test() {
  let (tx, rx) = Channel::new().pair();
  let mut executor = Executor::new();
  let receiver = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    let first = rx.receive().await.unwrap();
    let first_moment = Task::now();
    let second = rx.receive().await.unwrap();
    ([first, second], first_moment)
  });
  let sender = executor.spawn(async move {
    tx.send(1).await.unwrap();
    tx.send_with_options(
      2,
      SendOptions::new().timeout(Duration::from_nanos(10)),
    )
    .await
    .unwrap();
    Task::now()
  });
  executor.run();

  assert_eq!(receiver.output(), Some(([1, 2], Moment::from_nanos(10))),);
  assert_eq!(sender.output(), Some(Moment::from_nanos(10)));
}
