//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::Moment;

// Subtracting moments returns their elapsed duration.
#[test]
fn test() {
  let earlier = Moment::ZERO + Duration::from_secs(1);
  let later = earlier + Duration::from_millis(1250);
  assert_eq!(later - earlier, Duration::from_millis(1250));
}
