//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::SendOptions;

// SendOptions setters configure latency and timeout.
#[test]
fn test() {
  let latency = Duration::from_nanos(10);
  let timeout = Duration::from_nanos(20);
  let options = SendOptions::new().latency(latency).timeout(timeout);
  assert_eq!(options.latency, latency);
  assert_eq!(options.timeout, Some(timeout));
}
