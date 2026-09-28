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
use crate::poll;

/// Fans multiple message streams into one stream.
pub struct FanInBlock<T, E: Error + 'static = Infallible> {
  /// The endpoints to which the applications send messages.
  pub app_txs: Vec<Tx<T, E>>,

  /// The endpoint from which the network receives messages.
  pub net_rx: Rx<T, E>,
}

impl<T: 'static, E: Error + 'static> FanInBlock<T, E> {
  #[must_use]
  pub fn new(executor: &mut Executor, input_count: usize) -> Self {
    assert!(
      input_count > 0,
      "A FanInBlock must have at least one input.",
    );

    let mut app_txs = Vec::with_capacity(input_count);
    let mut app_rxs = Vec::with_capacity(input_count);
    for _ in 0..input_count {
      let (app_tx, app_rx) = Channel::new().fallible::<E>().pair();
      app_txs.push(app_tx);
      app_rxs.push(app_rx);
    }
    let (net_tx, net_rx) = Channel::new().fallible::<E>().pair();

    executor.spawn(async move {
      while !app_rxs.is_empty() {
        let i = poll(&app_rxs).await;
        match app_rxs[i].expect_receive() {
          Ok(message) => match net_tx.send(message).await {
            Ok(()) => {}
            Err(error) => match error.cause {
              ChannelError::Aborted(info) => {
                for app_rx in &app_rxs {
                  app_rx.abort_with(info.clone());
                }
                return;
              }
              ChannelError::Closed => return,
              ChannelError::Failed(error) => {
                for app_rx in &app_rxs {
                  app_rx.fail(error.clone());
                }
                return;
              }
              ChannelError::TimedOut => return,
            },
          },
          Err(ChannelError::Aborted(info)) => {
            net_tx.abort_with(info.clone());
            for app_rx in &app_rxs {
              app_rx.abort_with(info.clone());
            }
            return;
          }
          Err(ChannelError::Closed) => {
            let _ = app_rxs.remove(i);
          }
          Err(ChannelError::Failed(error)) => {
            net_tx.fail(error.clone());
            for app_rx in &app_rxs {
              app_rx.fail(error.clone());
            }
            return;
          }
          Err(ChannelError::TimedOut) => return,
        }
      }
    });

    Self { app_txs, net_rx }
  }
}

#[cfg(test)]
mod tests;
