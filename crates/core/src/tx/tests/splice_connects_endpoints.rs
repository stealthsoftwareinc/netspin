//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;
use crate::Splice;

// Splicing two channels connects their remaining endpoints.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (upstream_tx, upstream_rx) = Channel::new().pair();
  let (downstream_tx, downstream_rx) = Channel::new().pair();
  downstream_tx.splice(upstream_rx);
  executor.spawn(async move {
    upstream_tx.send(7).await.unwrap();
  });
  let output = executor
    .spawn(async move { downstream_rx.receive().await.unwrap() });
  executor.run();
  assert_eq!(output.output(), Some(7));
}
