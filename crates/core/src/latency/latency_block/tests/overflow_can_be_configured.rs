//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::latency::LatencyBlock;
use crate::latency::LatencyOverflow;

// A LatencyBlock's overflow behavior can be configured.
#[test]
fn test() {
  let block =
    LatencyBlock::<()>::new().overflow(LatencyOverflow::Backpressure);
  assert_eq!(block.overflow, LatencyOverflow::Backpressure);
}
