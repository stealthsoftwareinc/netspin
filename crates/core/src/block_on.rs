//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::RefCell;
use core::future::Future;
use std::sync::OnceLock;

use tokio::runtime::Builder;
use tokio::runtime::Handle;
use tokio::runtime::Runtime;

const MODULE: &str = module_path!();

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub(crate) fn handle() -> Handle {
  RUNTIME
    .get_or_init(|| {
      Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap_or_else(|e| {
          panic!(
            "{MODULE}::handle(): Error creating Tokio runtime: {e}."
          )
        })
    })
    .handle()
    .clone()
}

thread_local! {
  static HANDLE: RefCell<Option<Handle>> = const { RefCell::new(None) };
}

pub(crate) fn set_handle(handle: Option<Handle>) {
  HANDLE.with_borrow_mut(|x| *x = handle);
}

pub(crate) fn block_on<F: Future>(future: F) -> F::Output {
  HANDLE
    .with_borrow(Clone::clone)
    .unwrap_or_else(|| {
      panic!("{MODULE}::block_on(): HANDLE must not be None.")
    })
    .block_on(future)
}
