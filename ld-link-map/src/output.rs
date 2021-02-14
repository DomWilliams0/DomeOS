pub trait Sink {
    fn entry(&mut self, addr: u64, name: String);
}

/// Format:
/// [ 4 byte entry count ] followed by n of:
/// [ 8 byte address | 2 byte string len n | n bytes for string ]
pub struct PackedFormatSink(Vec<(u64, String)>);

#[derive(Default)]
pub struct StdoutSink(usize);

impl Sink for StdoutSink {
    fn entry(&mut self, addr: u64, name: String) {
        println!("{:4}) {:#x} = {}", self.0, addr, name);
        self.0 += 1;
    }
}

impl Default for PackedFormatSink {
    fn default() -> Self {
        Self(Vec::with_capacity(4096))
    }
}

impl Sink for PackedFormatSink {
    fn entry(&mut self, addr: u64, name: String) {
        // collect before sorting
        self.0.push((addr, name));
    }
}

impl PackedFormatSink {
    pub fn finish(mut self) -> Vec<u8> {
        self.0.sort_unstable_by_key(|(addr, _)| *addr);

        let capacity = 4+ // header
            self.0.len() * (
                8 + // addr
                2 // string len
            ) +
        self.0.iter().map(|(_, s)| s.len()).sum::<usize>(); // all string bytes
        let mut vec = Vec::with_capacity(capacity);

        vec.extend(&(self.0.len() as u32).to_le_bytes());
        for (addr, name) in self.0 {
            vec.extend(&addr.to_le_bytes());

            vec.extend(&(name.len() as u16).to_le_bytes());
            vec.extend(name.bytes());
        }

        assert_eq!(vec.len(), capacity);
        vec
    }
}
