//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Splice;
use crate::Task;

// Splicing retains the upstream channel's nonlocal latency behavior.
#[test]
fn test() {
  let latency = Duration::from_nanos(100);
  let (upstream_tx, upstream_rx) = Channel::new().local(false).pair();
  let (downstream_tx, downstream_rx) = Channel::new().pair();
  downstream_tx.splice(upstream_rx);
  let mut executor = Executor::new();
  let received = executor.spawn(async move {
    upstream_tx
      .send_with_options(7, SendOptions::new().latency(latency))
      .await
      .unwrap();
    let message = downstream_rx.receive().await.unwrap();
    (message, Task::now())
  });
  executor.run();
  assert_eq!(received.output(), Some((7, Moment::from_nanos(100))));
}
