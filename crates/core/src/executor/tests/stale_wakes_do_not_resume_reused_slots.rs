//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::RefCell;
use core::future::poll_fn;
use core::task::Poll;
use core::task::Waker;
use core::time::Duration;
use std::rc::Rc;

use crate::Executor;
use crate::Moment;
use crate::Task;

// A completed task's retained waker cannot resume a new task that uses
// the same slot, while the new task's own waker still works.
#[test]
fn test() {
  let mut executor = Executor::new();
  let output = executor.spawn(async {
    let original = Task::spawn(async {
      poll_fn(|cx| Poll::Ready((Task::key(), cx.waker().clone()))).await
    });
    Task::sleep(Duration::ZERO).await;
    let (old_key, old_waker) = original.output().unwrap();
    let waker = Rc::new(RefCell::new(None::<Waker>));
    let replacement = Task::spawn({
      let waker = waker.clone();
      async move {
        let new_key = Task::key();
        assert_eq!(new_key.slot, old_key.slot);
        assert_ne!(new_key.id, old_key.id);
        Task::park(|new_waker| {
          *waker.borrow_mut() = Some(new_waker.clone());
        })
        .await;
        Task::now()
      }
    });
    Task::sleep(Duration::ZERO).await;
    old_waker.wake_by_ref();
    Task::sleep(Duration::from_nanos(10)).await;
    let premature = replacement.output.get();
    assert_eq!(premature, None);
    waker.borrow_mut().take().unwrap().wake();
    Task::sleep(Duration::from_nanos(1)).await;
    replacement.output().unwrap()
  });
  executor.run();
  let moment = output.output();
  assert_eq!(moment, Some(Moment::from_nanos(10)));
}
