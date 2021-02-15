use byteorder::{ByteOrder, LittleEndian};

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug)]
pub struct Entry<'a> {
    pub address: u64,
    pub name: &'a str,
}

/// Panics if input is malformed
pub fn iter_entries(packed: &[u8]) -> impl Iterator<Item = Entry> + Clone + DoubleEndedIterator {
    let mut cursor = 0;

    macro_rules! read {
        ($ty:ty, $method:ident) => {{
            let sz = core::mem::size_of::<$ty>();
            let slice = &packed[cursor..cursor + sz];
            assert_eq!(slice.len(), sz, "malformed input at offset {}", cursor);

            let val = LittleEndian::$method(slice);
            cursor += sz;
            val
        }};
    }

    let count = read!(u32, read_u32);

    (0..count).map(move |_| {
        let addr = read!(u64, read_u64);
        let str_len = read!(u16, read_u16) as usize;

        let string = &packed[cursor..cursor + str_len];
        assert_eq!(
            string.len(),
            str_len,
            "malformed input at offset {}",
            cursor
        );
        cursor += str_len;

        let string = unsafe { core::str::from_utf8_unchecked(string) };

        Entry {
            address: addr,
            name: string,
        }
    })
}

pub fn resolve_entry<'e>(
    mut entries: impl Iterator<Item = Entry<'e>> + DoubleEndedIterator,
    addr: u64,
) -> Option<Entry<'e>> {
    // TODO binary search
    let mut last = None;
    entries.find_map(|e| {
        if e.address > addr {
            core::mem::take(&mut last)
        } else {
            last = Some(e);
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{PackedFormatSink, Sink};

    fn make_packed() -> Vec<u8> {
        let mut sink = PackedFormatSink::default();
        sink.entry(123, "epic".to_owned());
        sink.entry(456, "fantastic".to_owned());
        sink.entry(222, "awesome".to_owned());

        sink.entry(0xffffffff801052e0, "kernel_main".to_owned());

        sink.entry(
            0xffffffff8018e5f0,
            "core::num::<impl usize>::overflowing_mul".to_owned(),
        );
        sink.entry(
            0xffffffff8018e650,
            "domeos_kernel::panic::panic_handler::{{closure}}".to_owned(),
        );
        sink.entry(
            0xffffffff8018e780,
            "domeos_kernel::panic::backtrace".to_owned(),
        );
        sink.entry(
            0xffffffff8018ec30,
            "domeos_kernel::panic::backtrace::{{closure}}::{{closure}}".to_owned(),
        );
        sink.entry(
            0xffffffff8018ec80,
            "domeos_kernel::panic::backtrace::{{closure}}".to_owned(),
        );
        sink.entry(
            0xffffffff8018ee90,
            "<T as core::convert::From<T>>::from".to_owned(),
        );
        sink.finish()
    }

    #[test]
    fn parse_packed() {
        let packed = make_packed();
        let entries: Vec<_> = iter_entries(&packed).take(3).collect();

        assert_eq!(
            entries,
            vec![
                Entry {
                    address: 123,
                    name: "epic",
                },
                Entry {
                    address: 222, // is sorted by address
                    name: "awesome",
                },
                Entry {
                    address: 456,
                    name: "fantastic",
                },
            ]
        )
    }

    #[test]
    fn resolve() {
        let packed = make_packed();
        let entries = iter_entries(&packed);

        assert_eq!(
            resolve_entry(entries.clone(), 125).map(|e| e.name),
            Some("epic")
        );
        assert_eq!(resolve_entry(entries.clone(), 100), None);
        assert_eq!(
            resolve_entry(entries.clone(), 222).map(|e| e.name),
            Some("awesome")
        );

        assert_eq!(
            resolve_entry(entries.clone(), 0xffffffff8018eba0).map(|e| e.name),
            Some("domeos_kernel::panic::backtrace")
        );
    }

    #[test]
    #[should_panic]
    fn parse_malformed() {
        iter_entries(&[1, 2, 3]).for_each(|_| {});
    }
}
