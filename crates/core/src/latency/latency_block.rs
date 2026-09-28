//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::error::Error;

use crate::Attach;
use crate::Channel;
use crate::ChannelError;
use crate::Pump;
use crate::Rx;
use crate::SendOptions;
use crate::Spawn;
use crate::Splice;
use crate::Tx;
use crate::latency::LatencyModel;
use crate::latency::LatencyOverflow;
use crate::latency::NoLatency;
use crate::loss::LossModel;

/// Configures a buffered latency block with optional message loss.
pub struct LatencyBlock<T> {
  channel: Channel<T>,
  loss_model: Option<Box<dyn LossModel>>,
  model: Box<dyn LatencyModel>,
  overflow: LatencyOverflow,
}

impl<T> Clone for LatencyBlock<T> {
  fn clone(&self) -> Self {
    Self {
      channel: self.channel.clone(),
      loss_model: self.loss_model.clone(),
      model: self.model.clone(),
      overflow: self.overflow,
    }
  }
}

impl<T> LatencyBlock<T> {
  /// The default maximum total message size that may be in flight.
  pub const DEFAULT_CAPACITY: usize = 1000;

  /// Creates a latency block with the default settings.
  #[must_use]
  pub fn new() -> Self {
    Self {
      channel: Channel::new()
        .capacity(Some(Self::DEFAULT_CAPACITY))
        .local(false),
      loss_model: None,
      model: Box::new(NoLatency::new()),
      overflow: LatencyOverflow::default(),
    }
  }

  /// Sets the maximum permitted total message size.
  ///
  /// `None` means the buffer is unbounded.
  ///
  /// The default is [`DEFAULT_CAPACITY`].
  ///
  /// [`DEFAULT_CAPACITY`]: LatencyBlock::DEFAULT_CAPACITY
  #[must_use]
  pub fn capacity(mut self, capacity: Option<usize>) -> Self {
    self.channel = self.channel.capacity(capacity);
    self
  }

  /// Sets a loss model to apply before latency and buffer admission.
  /// Dropped messages do not consume buffer space or advance the
  /// latency model.
  /// By default, no loss model is applied.
  #[must_use]
  pub fn loss_model(mut self, model: impl LossModel + 'static) -> Self {
    self.loss_model = Some(Box::new(model));
    self
  }

  /// Sets the function that computes the size of a message.
  ///
  /// The default is a function that returns `1` for every message.
  #[must_use]
  pub fn message_size(
    mut self,
    message_size: impl Fn(&T) -> usize + 'static,
  ) -> Self {
    self.channel = self.channel.message_size(message_size);
    self
  }

  /// Sets the latency model.
  ///
  /// The default is [`NoLatency`].
  ///
  /// [`NoLatency`]: crate::latency::NoLatency
  #[must_use]
  pub fn model(mut self, model: impl LatencyModel + 'static) -> Self {
    self.model = Box::new(model);
    self
  }

  /// Sets the behavior when the latency buffer is full.
  ///
  /// The default is [`LatencyOverflow::TailDrop`].
  ///
  /// [`LatencyOverflow::TailDrop`]: LatencyOverflow::TailDrop
  #[must_use]
  pub fn overflow(mut self, overflow: LatencyOverflow) -> Self {
    self.overflow = overflow;
    self
  }
}

impl<T> Default for LatencyBlock<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: 'static, E: Error + 'static> Attach<Rx<T, E>>
  for LatencyBlock<T>
{
  type Surface = Rx<T, E>;

  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: Rx<T, E>,
  ) -> Self::Surface {
    let Self {
      channel,
      mut loss_model,
      mut model,
      overflow,
    } = self;
    let (output_tx, output_rx) = channel.fallible().pair();
    spawner.spawn(async move {
      loop {
        let message = match target.receive().await {
          Ok(message) => message,
          Err(ChannelError::Aborted(info)) => {
            output_tx.abort_with(info);
            return;
          }
          Err(ChannelError::Closed) => return,
          Err(ChannelError::Failed(error)) => {
            output_tx.fail(error);
            return;
          }
          Err(ChannelError::TimedOut) => return,
        };
        if loss_model.as_mut().is_some_and(|model| model.should_drop())
        {
          continue;
        }
        let latency = model.next_latency();
        let result = match overflow {
          LatencyOverflow::Backpressure => {
            let options = SendOptions::new().latency(latency);
            output_tx
              .send_with_options(message, options)
              .await
              .map(|_| ())
          }
          LatencyOverflow::TailDrop => {
            let options = SendOptions::new().latency(latency);
            output_tx
              .try_send_with_options(message, options)
              .map(|_| ())
          }
        };
        match result {
          Ok(()) => {}
          Err(error) => match error.cause {
            ChannelError::Aborted(info) => {
              target.abort_with(info);
              return;
            }
            ChannelError::Closed => return,
            ChannelError::Failed(error) => {
              target.fail(error);
              return;
            }
            ChannelError::TimedOut => return,
          },
        }
      }
    });
    output_rx
  }
}

impl<T: 'static, E: Error + 'static> Attach<(Rx<T, E>, Tx<T, E>)>
  for LatencyBlock<T>
{
  type Surface = ();

  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: (Rx<T, E>, Tx<T, E>),
  ) -> Self::Surface {
    let (input, output) = target;
    let output_is_spliceable = output.is_spliceable_downstream();
    let delayed = Self::attach(self, spawner, input);
    if output_is_spliceable {
      delayed.splice(output);
    } else {
      Pump::new().attach(spawner, (delayed, output));
    }
  }
}

impl<T: 'static, E: Error + 'static> Attach<Tx<T, E>>
  for LatencyBlock<T>
{
  type Surface = Tx<T, E>;

  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: Tx<T, E>,
  ) -> Self::Surface {
    let (a, b) = Channel::new().fallible().pair();
    Self::attach(self, spawner, (b, target));
    a
  }
}

#[cfg(test)]
mod tests;
