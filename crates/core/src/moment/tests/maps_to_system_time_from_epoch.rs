//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;
use std::time::UNIX_EPOCH;

use super::super::Moment;

// A moment maps to its elapsed duration past the given epoch.
#[test]
fn test() {
  let epoch = UNIX_EPOCH + Duration::from_secs(1_000);
  let moment = Moment::ZERO + Duration::from_nanos(123_456_789);
  assert_eq!(
    moment.to_system_time(epoch),
    epoch + Duration::from_nanos(123_456_789)
  );
}
