#[cfg(feature = "nom")]
#[macro_use]
mod nom;

make_add_macro! {
    name: add_borrow;
    type: ::std::borrow::Borrow<[u8]>;
    body: {
        fn borrow(&self) -> &[u8] {
            self.as_ref()
        }
    }
}

make_add_macro! {
    name: add_read;
    type: ::std::io::Read;
    body: {
        fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
            if self.remaining() > buf.len() {
                self.next_bytes(buf)?;
                Ok(buf.len())
            } else {
                let read = self.remaining();
                for i in 0..read {
                    buf[i] = self.next_u8()?;
                }
                Ok(read)
            }
        }
    }
}

make_add_macro! {
    name: add_seek;
    type: ::std::io::Seek;
    body: {
        fn seek(&mut self, pos: ::std::io::SeekFrom) -> ::std::io::Result<u64> {
            match pos {
                ::std::io::SeekFrom::Start(to) => self.advance_to(to as usize)?,
                ::std::io::SeekFrom::Current(by) => self.advance_to(
                    (self.current_offset() as i64 + by) as usize)?,
                ::std::io::SeekFrom::End(point) => self.advance_to(
                    (self.upper_offset_limit() as i64 - point) as usize)?,
            };
            Ok(self.current_offset() as u64)
        }
    }
}

make_add_macro! {
    name: add_bufread;
    type: ::std::io::BufRead;
    body: {
        fn fill_buf(&mut self) -> ::std::io::Result<&[u8]> {
            if self.remaining() >= 4096 {
                Ok(self.subseq(self.current_offset(), 4096)?)
            } else {
                Ok(self.subseq(self.current_offset(), self.remaining())?)
            }
        }

        fn consume(&mut self, amt: usize) {
            if !self.is_empty() {
                if self.remaining() < amt {
                    self.advance_by(self.remaining() as isize).unwrap();
                } else {
                    self.advance_by(amt as isize).unwrap();
                }
            }
        }
    }
}
