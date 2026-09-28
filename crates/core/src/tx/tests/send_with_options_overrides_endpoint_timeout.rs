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

// A per-call timeout overrides the endpoint's sending timeout.
#[test]
fn test() {
  let (tx, rx) = Channel::new().pair();
  tx.set_send_timeout(Some(Duration::from_nanos(5)));
  let mut executor = Executor::new();
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(7)).await;
    assert_eq!(rx.receive().await.unwrap(), 1);
    assert_eq!(rx.receive().await.unwrap(), 2);
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

  assert_eq!(sender.output(), Some(Moment::from_nanos(7)));
}
