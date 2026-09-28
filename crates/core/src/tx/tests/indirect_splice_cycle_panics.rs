//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Splice;

// Splicing endpoints already connected by prior splices panics.
#[test]
#[should_panic(expected = "Splicing channels must not create a cycle.")]
fn test() {
  let (a, b) = Channel::<()>::new().pair();
  let (c, d) = Channel::new().pair();
  c.splice(b);
  a.splice(d);
}
