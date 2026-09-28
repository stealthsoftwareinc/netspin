//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::latency::LatencyBlock;
use crate::latency::LatencyOverflow;

// LatencyBlock constructors use a 1,000-message capacity, no latency,
// and tail-drop overflow behavior.
#[test]
fn test() {
  for mut block in
    [LatencyBlock::<()>::new(), LatencyBlock::<()>::default()]
  {
    assert_eq!(block.channel.capacity, Some(1000));
    assert_eq!(
      block.channel.capacity,
      Some(LatencyBlock::<()>::DEFAULT_CAPACITY)
    );
    assert!(block.channel.message_size.is_none());
    assert_eq!(block.model.next_latency(), Duration::ZERO);
    assert_eq!(block.overflow, LatencyOverflow::TailDrop);
  }
}
