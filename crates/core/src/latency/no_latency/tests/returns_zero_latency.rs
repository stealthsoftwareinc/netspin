//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::latency::LatencyModel;
use crate::latency::NoLatency;

// NoLatency returns zero latency for every message.
#[test]
fn test() {
  let mut model = NoLatency::new();
  assert_eq!(model.next_latency(), Duration::ZERO);
  assert_eq!(model.next_latency(), Duration::ZERO);
}
