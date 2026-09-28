//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::latency::LatencyBlock;

// A LatencyBlock buffer can be configured as unbounded.
#[test]
fn test() {
  let block = LatencyBlock::<()>::new().capacity(None);
  assert_eq!(block.channel.capacity, None);
}
