use core::marker::PhantomData;

use byte::ctx::Bytes;
use byte::{
    check_len,
    ctx::{Endian, LE},
    BytesExt, TryRead,
};

use crate::address::{Address, FileOffset};
use crate::error::{PeError, PeResult};
use alloc::string::String;

#[derive(Clone)]
pub struct Cursor<'b> {
    buf: &'b [u8],

    /// Next byte to read
    cursor: usize,

    /// The offset of this buffer in the file, set once on creation
    file_pointer: usize,
}

/// # Safety
/// Contains no references or pointers, and a reference to this type can be safely constructed
/// from a &[u8] in [Cursor::read_reference]
pub unsafe trait Reinterpret {}

impl<'b> From<&'b [u8]> for Cursor<'b> {
    fn from(bytes: &'b [u8]) -> Self {
        Self {
            buf: bytes,
            cursor: 0,
            file_pointer: 0,
        }
    }
}

impl<'b> Cursor<'b> {
    pub fn read<T: TryRead<'b, Endian>>(&mut self) -> PeResult<T> {
        self.buf
            .read_with(&mut self.cursor, LE)
            .map_err(|e| e.into())
    }

    pub fn read_slice<'s: 'b, S>(&mut self, n: usize) -> PeResult<&'s [S]> {
        // Windows and PEs are always little endian
        #[cfg(not(target_endian = "little"))]
        return Err(PeError::UnsupportedEndianness);

        let size = core::mem::size_of::<S>() * n;
        let slice: &[u8] = self
            .buf
            .read_with(&mut self.cursor, Bytes::Len(size))
            .map_err(PeError::from)?;

        unsafe {
            let slice_ptr = slice.as_ptr() as *const S;
            Ok(core::slice::from_raw_parts(slice_ptr, n))
        }
    }
    pub fn read_reference<T: Reinterpret>(&mut self) -> PeResult<&'b T> {
        let size = core::mem::size_of::<T>();
        let slice = self.read_slice::<u8>(size)?;

        // safety: lifetime parameter preserved
        let val: &'b T = unsafe { &*(slice.as_ptr() as *const T) };

        Ok(val)
    }

    pub fn consume(mut self) -> &'b [u8] {
        self.read_until(self.buf.len()).unwrap() // cant fail
    }

    /// Does not advance cursor
    pub fn read_at_offset<T: TryRead<'b, Endian>>(&self, offset: usize) -> PeResult<T> {
        let mut offset = offset;
        self.buf.read_with(&mut offset, LE).map_err(|e| e.into())
    }

    /// Excludes offset from returned slice
    pub fn read_until(&mut self, offset: usize) -> PeResult<&'b [u8]> {
        if offset <= self.cursor {
            Ok(&[])
        } else {
            check_len(&self.buf, offset)?;
            let slice = &self.buf[self.cursor..offset];
            self.cursor = offset;
            Ok(slice)
        }
    }

    pub fn skip(&mut self, n: usize) -> PeResult<()> {
        let new_offset = self.cursor + n;
        check_len(&self.buf, new_offset)?;
        self.cursor = new_offset;
        Ok(())
    }

    pub fn read_magic<T: TryRead<'b, Endian> + Eq + Into<u64>>(
        &mut self,
        expected_magic: T,
        magic: &'static str,
    ) -> PeResult<()> {
        self.buf
            .read_with(&mut self.cursor, LE)
            .map_err(|e| e.into())
            .and_then(|val: T| {
                if val == expected_magic {
                    Ok(())
                } else {
                    Err(PeError::IncorrectMagic {
                        magic,
                        expected: expected_magic.into(),
                        value: val.into(),
                    })
                }
            })
    }

    pub fn read_null_terminated_str(&mut self) -> PeResult<&'b str> {
        // clone to peek ahead until null byte
        let nul_idx = self
            .clone()
            .iter::<u8>()
            .position(|b| b == 0)
            .ok_or(PeError::MissingStringNullByte)?;

        let string_bytes = self.read_slice(nul_idx)?;
        core::str::from_utf8(string_bytes).map_err(PeError::InvalidString)
    }

    pub fn read_utf16_str(&mut self, bytes: usize) -> PeResult<String> {
        let string_bytes = self.read_slice(bytes / core::mem::size_of::<u16>())?;
        let mut err = None;
        let string = core::char::decode_utf16(string_bytes.iter().copied())
            .map(|result| match result {
                Ok(c) => c,
                Err(e) => {
                    err = Some(e);
                    core::char::REPLACEMENT_CHARACTER
                }
            })
            .collect::<String>();

        if let Some(err) = err {
            Err(PeError::InvalidStringUtf16(err))
        } else {
            Ok(string)
        }
    }

    pub fn restart(&mut self) {
        self.cursor = 0;
    }

    pub fn position(&self) -> usize {
        self.cursor
    }

    /// Absolute pointer in the file
    pub fn file_position(&self) -> FileOffset {
        FileOffset::with_usize(self.cursor + self.file_pointer)
    }

    pub fn seek_to(&self, offset: usize) -> PeResult<Self> {
        check_len(self.buf, offset)?;

        let mut clone = self.clone();
        clone.cursor = offset;
        Ok(clone)
    }

    pub fn sub_buffer(&mut self, length: usize) -> PeResult<Self> {
        let sub_buffer = Self::new(self.buf, FileOffset::with_usize(self.cursor), length)?;
        self.cursor += length;
        Ok(sub_buffer)
    }

    pub fn new(bytes: &'b [u8], offset: FileOffset, length: usize) -> PeResult<Self> {
        let end = (offset + length).into_usize();
        check_len(bytes, end)?;

        let sub_slice = &bytes[offset.into_usize()..end];
        Ok(Self {
            buf: sub_slice,
            cursor: 0,
            file_pointer: offset.into_usize(),
        })
    }

    pub fn iter<T: TryRead<'b, Endian>>(&mut self) -> BufferIter<'_, 'b, T> {
        BufferIter {
            buffer: self,
            _phantom: PhantomData,
        }
    }
}

pub struct BufferIter<'a, 'b, T: TryRead<'b, Endian>> {
    buffer: &'a mut Cursor<'b>,
    _phantom: PhantomData<T>,
}

impl<'a, 'b, T: TryRead<'b, Endian>> Iterator for BufferIter<'a, 'b, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.read().ok()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn read_until() {
        let numbers = (0u8..12).collect::<Vec<_>>();
        let mut buf = Cursor::from(numbers.as_slice());

        // read from beginning
        assert!(matches!(buf.read_until(4), Ok(&[0u8, 1, 2, 3])));

        // advance 2
        let _: u16 = buf.read().unwrap();

        // read from new offset
        assert!(matches!(buf.read_until(10), Ok(&[6, 7, 8, 9])));

        // read until index already read, should be empty
        assert!(matches!(buf.read_until(8), Ok(&[])));

        // too far
        assert!(buf.read_until(20).is_err());
    }

    #[test]
    fn sub_buffer() {
        let numbers = (0u8..12).collect::<Vec<_>>();
        let mut buf = Cursor::from(numbers.as_slice());

        buf.read_until(5).unwrap();
        assert_eq!(buf.cursor, 5);

        // new sub_buffer
        assert!(buf.sub_buffer(100).is_err());

        let mut subbuf = buf.sub_buffer(3).unwrap();

        // can read 2 from it
        assert!(subbuf.read::<u16>().is_ok());

        // but not 2 more as its length is limited to 3
        assert!(subbuf.read::<u16>().is_err());
    }

    #[test]
    fn new() {
        let numbers = (0u8..12).collect::<Vec<_>>();
        let mut buf = Cursor::new(numbers.as_slice(), FileOffset::new(4u64), 3).unwrap();

        assert_eq!(buf.cursor, 0);
        assert_eq!(buf.read::<u8>().unwrap(), 4);
        assert_eq!(buf.read::<u8>().unwrap(), 5);
        assert_eq!(buf.read::<u8>().unwrap(), 6);
        assert!(buf.read::<u8>().is_err());
    }

    #[test]
    fn slice() {
        let numbers = vec![1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0];
        let mut buf = Cursor::from(numbers.as_slice());

        let ints = buf.read_slice::<i32>(3).unwrap();
        assert_eq!(ints, [1, 2, 3]);
    }
}
