//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Splice;

// Splicing a downstream channel with a message split function panics.
#[test]
#[should_panic(
  expected = "Splicing a channel requires it to have no message split \
              function."
)]
fn test() {
  let (downstream_tx, _downstream_rx) =
    Channel::<()>::new().message_split(|_, _| None).pair();
  let (_upstream_tx, upstream_rx) = Channel::new().pair();
  downstream_tx.splice(upstream_rx);
}
