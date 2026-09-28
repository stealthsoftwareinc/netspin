//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::latency::LatencyOverflow;

// The default latency overflow behavior is tail drop.
#[test]
fn test() {
  assert_eq!(LatencyOverflow::default(), LatencyOverflow::TailDrop);
}
