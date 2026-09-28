//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;
use crate::Splice;

// Splicing a channel with an active receiver panics.
#[test]
#[should_panic(
  expected = "Splicing a channel requires it to have no active \
              receivers."
)]
fn test() {
  let mut executor = Executor::new();
  let (_upstream_tx, upstream_rx) = Channel::<()>::new().pair();
  let (downstream_tx, downstream_rx) = Channel::new().pair();
  executor.spawn(async move {
    downstream_rx.receive().await.unwrap();
  });
  executor.spawn(async move {
    downstream_tx.splice(upstream_rx);
  });
  executor.run();
}
