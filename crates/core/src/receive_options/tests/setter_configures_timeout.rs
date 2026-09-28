//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::ReceiveOptions;

// ReceiveOptions::timeout() configures the timeout.
#[test]
fn test() {
  let timeout = Duration::from_nanos(10);
  let options = ReceiveOptions::new().timeout(timeout);
  assert_eq!(options.timeout, Some(timeout));
}
