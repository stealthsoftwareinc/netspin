//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::latency::FixedLatency;
use crate::latency::LatencyBlock;

// A configured latency model provides the block's per-message latency.
#[test]
fn test() {
  let latency = Duration::from_millis(50);
  let mut block =
    LatencyBlock::<()>::new().model(FixedLatency::new(latency));
  assert_eq!(block.model.next_latency(), latency);
}
