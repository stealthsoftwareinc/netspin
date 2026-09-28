//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::Moment;

// Adding a duration by assignment advances the moment.
#[test]
fn test() {
  let mut moment = Moment::ZERO + Duration::from_secs(1);
  moment += Duration::from_millis(250);
  assert_eq!(moment - Moment::ZERO, Duration::from_millis(1250));
}
