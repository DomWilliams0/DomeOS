pub trait Sink {
    fn entry(&mut self, addr: u64, name: String);
}
#[derive(Default)]
pub struct StdoutSink(usize);

impl Sink for StdoutSink {
    fn entry(&mut self, addr: u64, name: String) {
        println!("{:4}) {:#x} = {}", self.0, addr, name);
        self.0 += 1;
    }
}

/// Format:
/// [ 4 byte entry count ] followed by n of:
/// [ 8 byte address | 2 byte string len n | n bytes for string ]
pub struct PackedFormatSink(Vec<(u64, String)>);

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

#[cfg(feature = "std")]
pub fn parse_text(input: impl std::io::BufRead, sink: &mut impl Sink) {
    let mut lines = input
        .lines()
        .map(|line| line.expect("failed to read line"))
        .filter(|line| !line.is_empty());

    // skip to the start
    lines
        .position(|line| line == "Linker script and memory map")
        .expect("malformed input");

    const SECTIONS: [&str; 2] = [".text", ".boot"];

    enum State {
        Waiting,
        GotName(String),
    }

    let mut state = State::Waiting;
    for line in lines {
        match &mut state {
            State::Waiting => {
                let line = line.trim();
                let words = line.split_whitespace().count();
                if words == 1 {
                    if let Some(&section) = SECTIONS.iter().find(|s| line.starts_with(*s)) {
                        let name = &line[section.len() + 1..];
                        state = State::GotName(format!("{:#?}", rustc_demangle::demangle(name)));
                    }
                }
            }
            State::GotName(name) => {
                // address should be first thing on following line
                let addr = line
                    .trim()
                    .split_whitespace()
                    .next()
                    .and_then(|word| u64::from_str_radix(word.trim_start_matches("0x"), 16).ok())
                    .unwrap_or_else(|| panic!("address expected but got line {:?}", line));

                sink.entry(addr, std::mem::take(name));
                state = State::Waiting;
            }
        }
    }
}
