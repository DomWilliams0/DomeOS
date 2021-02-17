use ld_link_map::text::{PackedFormatSink, StdoutSink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn main() {
    let mut args = std::env::args_os();
    let exe = args.next().unwrap();
    let (path, rip) = args.next().zip(args.next()).unwrap_or_else(|| {
        panic!(
            "usage: {} create <packed file> <rip to resolve>",
            Path::new(&exe).display()
        )
    });

    let rip = rip
        .to_str()
        .and_then(|s| u64::from_str_radix(s, 16).ok())
        .expect("invalid rip, should be hex address");

    let packed = std::fs::read(path).expect("failed to read input file");

    let entries = ld_link_map::packed::iter_entries(&packed);
    match ld_link_map::packed::resolve_entry(entries, rip) {
        Some(entry) => println!(
            "resolved to function {} at {:#x}",
            entry.name, entry.address
        ),
        None => println!("function not found"),
    }
}
