//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Splice;

// Splicing a downstream channel with a custom message size function
// panics.
#[test]
#[should_panic(
  expected = "Splicing a channel requires it to have no message size \
              function."
)]
fn test() {
  let (downstream_tx, _downstream_rx) =
    Channel::<()>::new().message_size(|_| 1).pair();
  let (_upstream_tx, upstream_rx) = Channel::new().pair();
  downstream_tx.splice(upstream_rx);
}
