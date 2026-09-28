//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

// This example implements FizzBuzz using three tasks:
//
// * `fizz()`, which pings a channel every 3 ticks.
// * `buzz()`, which pings a channel every 5 ticks.
// * `print()`, which runs every tick and writes output.
//
// Each tick, `print()` writes either the tick number, `"Fizz"`,
// `"Buzz"`, or `"FizzBuzz"` depending on which pings were received.
// There are no divisibility checks.
//
// This example subtly depends on the tiebreaking guarantee of the
// executor: when two tasks are equivalently furthest-behind, the task
// that has been suspended the longest will take priority. For example,
// `fizz()` and `print()` will both be suspended for tick 3, but the
// executor will resume `fizz()` first because it will have been
// suspended for longer. This means the ping will be ready once
// `print()` is resumed.
//
// You can run this example as follows:
//
//     cargo run --example fizzbuzz

use core::time::Duration;

use netspin_core::Channel;
use netspin_core::Executor;
use netspin_core::Rx;
use netspin_core::Task;
use netspin_core::Tx;

async fn fizz(tx: Tx<()>) {
  Task::sleep(Duration::from_nanos(3)).await;
  while Task::now().to_duration() <= Duration::from_nanos(100) {
    tx.send(()).await.unwrap();
    Task::sleep(Duration::from_nanos(3)).await;
  }
}

async fn buzz(tx: Tx<()>) {
  Task::sleep(Duration::from_nanos(5)).await;
  while Task::now().to_duration() <= Duration::from_nanos(100) {
    tx.send(()).await.unwrap();
    Task::sleep(Duration::from_nanos(5)).await;
  }
}

async fn print(fizz_rx: Rx<()>, buzz_rx: Rx<()>) {
  Task::sleep(Duration::from_nanos(1)).await;
  while Task::now().to_duration() <= Duration::from_nanos(100) {
    let fizzed = fizz_rx.try_receive().unwrap().is_some();
    let buzzed = buzz_rx.try_receive().unwrap().is_some();
    if fizzed {
      print!("Fizz");
    }
    if buzzed {
      print!("Buzz");
    }
    if !fizzed && !buzzed {
      print!("{}", Task::now().to_duration().as_nanos());
    }
    println!();
    Task::sleep(Duration::from_nanos(1)).await;
  }
}

fn main() {
  let mut executor = Executor::new();
  let (fizz_tx, fizz_rx) = Channel::new().pair();
  let (buzz_tx, buzz_rx) = Channel::new().pair();
  executor.spawn(fizz(fizz_tx));
  executor.spawn(buzz(buzz_tx));
  executor.spawn(print(fizz_rx, buzz_rx));
  executor.run();
}
