//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;

// Sending timeouts default to none and normalize zero to none.
#[test]
fn test() {
  let (tx, _rx) = Channel::<()>::new().pair();

  assert_eq!(tx.send_timeout(), None);
  tx.set_send_timeout(Some(Duration::from_nanos(7)));
  assert_eq!(tx.send_timeout(), Some(Duration::from_nanos(7)));
  tx.set_send_timeout(Some(Duration::ZERO));
  assert_eq!(tx.send_timeout(), None);
  tx.set_send_timeout(Some(Duration::from_nanos(8)));
  tx.set_send_timeout(None);
  assert_eq!(tx.send_timeout(), None);
}
