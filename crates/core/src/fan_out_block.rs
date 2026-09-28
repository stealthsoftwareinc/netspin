//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::convert::Infallible;
use core::error::Error;

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::Rx;
use crate::Tx;

/// Fans one message stream out to multiple streams.
///
/// Messages are sent to the outputs in ascending index order.
/// Each send waits for handoff capacity, so an earlier output can
/// backpressure all later outputs.
pub struct FanOutBlock<T, E: Error + 'static = Infallible> {
  /// The endpoint to which the application sends messages.
  pub app_tx: Tx<T, E>,

  /// The endpoints from which the networks receive messages.
  pub net_rxs: Vec<Rx<T, E>>,
}

impl<T: Clone + 'static, E: Error + 'static> FanOutBlock<T, E> {
  /// Creates a fan-out block with `output_count` handoff outputs.
  #[must_use]
  pub fn new(executor: &mut Executor, output_count: usize) -> Self {
    assert!(
      output_count > 0,
      "A FanOutBlock must have at least one output.",
    );
    let (app_tx, app_rx) = Channel::<T>::new().fallible::<E>().pair();
    let mut net_txs = Vec::with_capacity(output_count);
    let mut net_rxs = Vec::with_capacity(output_count);
    for _ in 0..output_count {
      let (net_tx, net_rx) = Channel::new().fallible::<E>().pair();
      net_txs.push(net_tx);
      net_rxs.push(net_rx);
    }
    let last_net_tx = net_txs
      .pop()
      .expect("A FanOutBlock must have at least one output.");
    executor.spawn(async move {
      loop {
        let message = match app_rx.receive().await {
          Ok(message) => message,
          Err(ChannelError::Aborted(info)) => {
            for net_tx in &net_txs {
              net_tx.abort_with(info.clone());
            }
            last_net_tx.abort_with(info);
            return;
          }
          Err(ChannelError::Closed) => return,
          Err(ChannelError::Failed(error)) => {
            for net_tx in &net_txs {
              net_tx.fail(error.clone());
            }
            last_net_tx.fail(error);
            return;
          }
          Err(ChannelError::TimedOut) => return,
        };
        for net_tx in &net_txs {
          match net_tx.send(message.clone()).await {
            Ok(()) => {}
            Err(error) => match error.cause {
              ChannelError::Aborted(info) => {
                app_rx.abort_with(info.clone());
                for net_tx in &net_txs {
                  net_tx.abort_with(info.clone());
                }
                last_net_tx.abort_with(info);
                return;
              }
              ChannelError::Closed => return,
              ChannelError::Failed(error) => {
                app_rx.fail(error.clone());
                for net_tx in &net_txs {
                  net_tx.fail(error.clone());
                }
                last_net_tx.fail(error);
                return;
              }
              ChannelError::TimedOut => return,
            },
          }
        }
        match last_net_tx.send(message).await {
          Ok(()) => {}
          Err(error) => match error.cause {
            ChannelError::Aborted(info) => {
              app_rx.abort_with(info.clone());
              for net_tx in &net_txs {
                net_tx.abort_with(info.clone());
              }
              last_net_tx.abort_with(info);
              return;
            }
            ChannelError::Closed => return,
            ChannelError::Failed(error) => {
              app_rx.fail(error.clone());
              for net_tx in &net_txs {
                net_tx.fail(error.clone());
              }
              last_net_tx.fail(error);
              return;
            }
            ChannelError::TimedOut => return,
          },
        }
      }
    });
    Self { app_tx, net_rxs }
  }
}

#[cfg(test)]
mod tests;
