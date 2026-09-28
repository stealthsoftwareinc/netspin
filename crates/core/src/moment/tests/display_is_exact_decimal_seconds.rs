//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::Moment;

// A moment displays as exact decimal seconds with no unnecessary
// fractional zeroes.
#[test]
fn test() {
  for (nanos, expected) in [
    (0, "0"),
    (1, "0.000000001"),
    (1_000_000, "0.001"),
    (100_000_000, "0.1"),
    (1_001_002_030, "1.00100203"),
    (12_000_000_000, "12"),
    (u64::MAX, "18446744073.709551615"),
  ] {
    let moment = Moment::ZERO + Duration::from_nanos(nanos);
    assert_eq!(moment.to_string(), expected);
  }
}
