//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;
use crate::Splice;

// Splicing from a receiver connects the two remaining endpoints.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (a, b) = Channel::new().pair();
  let (c, d) = Channel::new().pair();
  b.splice(c);
  executor.spawn(async move {
    a.send(7).await.unwrap();
  });
  let output =
    executor.spawn(async move { d.receive().await.unwrap() });
  executor.run();
  assert_eq!(output.output(), Some(7));
}
