//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::Moment;

// Subtracting a later moment from an earlier moment panics.
#[test]
#[should_panic(expected = "Moment subtraction would be negative.")]
fn test() {
  let earlier = Moment::ZERO;
  let later = earlier + Duration::from_nanos(1);
  let _ = earlier - later;
}
