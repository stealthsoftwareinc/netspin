//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use std::cell::Cell;
use std::io;
use std::io::Write;
use std::rc::Rc;

use crate::PanickingWriter;

struct Writer {
  flushed: Rc<Cell<bool>>,
}

impl Write for Writer {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    Ok(buf.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    self.flushed.set(true);
    Ok(())
  }
}

#[test]
fn test() {
  let flushed = Rc::new(Cell::new(false));
  {
    let _writer = PanickingWriter::new(
      Writer {
        flushed: flushed.clone(),
      },
      "output",
    );
  }
  assert!(flushed.get());
}
