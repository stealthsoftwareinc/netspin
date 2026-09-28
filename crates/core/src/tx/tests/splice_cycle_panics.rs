//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Splice;

// Splicing the two endpoints of one channel panics.
#[test]
#[should_panic(expected = "Splicing channels must not create a cycle.")]
fn test() {
  let (tx, rx) = Channel::<()>::new().pair();
  tx.splice(rx);
}
