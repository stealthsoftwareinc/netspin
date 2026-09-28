//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::rc::Rc;

use crate::PanickingWriter;

struct Writer {
  output: Rc<RefCell<Vec<u8>>>,
}

impl Write for Writer {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.output.borrow_mut().extend_from_slice(buf);
    Ok(buf.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

#[test]
fn test() {
  let output = Rc::new(RefCell::new(Vec::new()));
  let mut writer = PanickingWriter::new(
    Writer {
      output: output.clone(),
    },
    "output",
  );

  writer.write_all(b"hello").unwrap();

  assert_eq!(*output.borrow(), b"hello");
}
