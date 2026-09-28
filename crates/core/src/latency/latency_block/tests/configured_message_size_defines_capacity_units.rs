//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::latency::LatencyBlock;

// The configured message-size function defines the units of the
// latency buffer capacity.
#[test]
fn test() {
  let block = LatencyBlock::<Vec<u8>>::new()
    .capacity(Some(10))
    .message_size(Vec::len);
  assert_eq!(block.channel.capacity, Some(10));
  assert_eq!(
    (block.channel.message_size.as_ref().unwrap())(&vec![1, 2, 3]),
    3,
  );
}
