use ld_link_map::text::{PackedFormatSink, StdoutSink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn main() {
    let mut args = std::env::args_os();
    let exe = args.next().unwrap();
    let path = args.next().unwrap_or_else(|| {
        panic!(
            "usage: {} <link map> [output file]",
            Path::new(&exe).display()
        )
    });

    let out = args.next();

    let file = File::open(path).expect("failed to open link map");
    let reader = BufReader::new(file);

    match out {
        None => {
            // parse and print
            let mut sink = StdoutSink::default();
            ld_link_map::text::parse_text(reader, &mut sink);
            println!("(pass second arg to write binary file instead of printing)");
        }
        Some(path) => {
            // parse and write to packed binary file
            let mut sink = PackedFormatSink::default();
            ld_link_map::text::parse_text(reader, &mut sink);

            let blob = sink.finish();
            let len = blob.len();
            std::fs::write(&path, blob).expect("failed to write binary file");
            println!("wrote {} bytes to {:?}", len, path);
        }
    }
}
