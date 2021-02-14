use crate::output::Sink;

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
                let addr = line.trim().split_whitespace().next().and_then(|word| {
                    u64::from_str_radix(word.trim_start_matches("0x"), 16).ok()
                }).unwrap_or_else(|| panic!("address expected but got line {:?}", line));

                sink.entry(addr, std::mem::take(name));
                state = State::Waiting;
            }
        }
    }
}
