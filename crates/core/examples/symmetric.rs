//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

// In this example, Alice and Bob send messages to each other over
// unbounded channels. Their behavior is symmetric: randomly choose
// whether to send or try receive a message, sleep for a random amount
// of time, and repeat a total of 999 times.
//
// Note that their behavior is cyclically dependent in that how often
// Bob tries to receive affects how much Alice's sends pile up in the
// channel, and vice versa.
//
// By default, both Alice and Bob choose to send with 50% probability.
// You can change their probabilities by specifying two command-line
// arguments. Here's some interesting variants to try:
//
//     cargo run --example symmetric
//     cargo run --example symmetric 0.0 0.0
//     cargo run --example symmetric 0.0 1.0
//     cargo run --example symmetric 1.0 0.0
//     cargo run --example symmetric 1.0 1.0
//     cargo run --example symmetric 0.9 0.9

use core::time::Duration;

use rand::RngExt;

use netspin_core::Channel;
use netspin_core::Executor;
use netspin_core::Moment;
use netspin_core::Rx;
use netspin_core::Task;
use netspin_core::Tx;

// Runs either Alice or Bob (the behavior is symmetric).
async fn party(
  tx: Tx<()>,
  rx: Rx<()>,
  prob: f64,
) -> (Moment, u64, u64) {
  let mut rng = Task::rng();
  let mut tx_count = 0;
  let mut rx_count = 0;
  for _ in 0..999 {
    if rng.random_bool(prob) {
      tx.send(()).await.unwrap();
      tx_count += 1;
    } else if rx.try_receive().unwrap().is_some() {
      rx_count += 1;
    }
    Task::sleep(Duration::from_nanos(rng.random_range(0..10))).await;
  }
  (Task::now(), tx_count, rx_count)
}

fn report((clock, tx_count, rx_count): (Moment, u64, u64), name: &str) {
  println!(
    "{:6} clock = {:4}, tx_count = {:3}, rx_count = {:3}, rx_tries = {:3}",
    format!("{name}:"),
    clock.to_duration().as_nanos(),
    tx_count,
    rx_count,
    999 - tx_count,
  );
}

fn parse_prob(arg: Option<String>) -> f64 {
  arg
    .map_or(0.5, |s| s.parse::<f64>().expect("Expected a probability"))
    .clamp(0.0, 1.0)
}

fn main() {
  let mut args = std::env::args().skip(1);
  let a_prob = parse_prob(args.next());
  let b_prob = parse_prob(args.next());
  let mut executor = Executor::new();
  let (a2b_tx, a2b_rx) =
    Channel::new().capacity(None).local(false).pair();
  let (b2a_tx, b2a_rx) =
    Channel::new().capacity(None).local(false).pair();
  let a = executor.spawn(party(a2b_tx, b2a_rx, a_prob));
  let b = executor.spawn(party(b2a_tx, a2b_rx, b_prob));
  executor.run();
  report(a.output().unwrap(), "Alice");
  report(b.output().unwrap(), "Bob");
}
