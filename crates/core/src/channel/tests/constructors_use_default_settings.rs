//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::ChannelOrder;

// Channel constructors use the documented default settings.
#[test]
fn test() {
  for channel in [Channel::<()>::new(), Channel::<()>::default()] {
    assert_eq!(channel.capacity, Some(1));
    assert!(channel.message_size.is_none());
    assert!(channel.message_split.is_none());
    assert!(channel.local);
    assert_eq!(channel.order, ChannelOrder::Ordered);
  }
}
