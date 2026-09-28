//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::latency::FixedLatency;
use crate::latency::LatencyModel;

// FixedLatency returns its configured latency for every message.
#[test]
fn test() {
  let latency = Duration::from_millis(50);
  let mut model = FixedLatency::new(latency);
  assert_eq!(model.next_latency(), latency);
  assert_eq!(model.next_latency(), latency);
}
