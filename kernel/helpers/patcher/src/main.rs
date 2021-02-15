use std::fs::OpenOptions;
use std::path::Path;

const MARKER: u32 = 0xbeef_face;
const MARKER_BYTES: [u8; 4] = u32::to_le_bytes(MARKER);

#[cfg(not(test))]
const MIN_MARKER_BYTES: usize = 1024;

#[cfg(test)]
const MIN_MARKER_BYTES: usize = 8;

fn main() {
    let mut args = std::env::args_os();
    let exe = args.next().unwrap();
    let (binary_to_patch, blob) = match (args.next(), args.next()) {
        (Some(a), Some(b)) => (a, b),
        _ => panic!(
            "usage: {} [binary file to patch] [source file to patch in]",
            Path::new(&exe).display()
        ),
    };

    let blob = std::fs::read(blob).expect("failed to read patching blob");

    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(false)
        .open(binary_to_patch)
        .expect("failed to open file");
    let mut mapped = unsafe { memmap::MmapMut::map_mut(&file).expect("failed to map file") };

    patch_marker_range(&mut mapped, &blob);
    println!("patching done")
}

fn patch_marker_range(file: &mut [u8], to_patch: &[u8]) -> (usize, usize) {
    let marker_size = core::mem::size_of_val(&MARKER);

    let mut search_start = 0;
    let (base, len) = loop {
        let base = file[search_start..]
            .chunks_exact(marker_size)
            .position(|val| val == &MARKER_BYTES)
            .map(|x| x * marker_size)
            .unwrap_or_else(|| panic!("marker not found ({:#x})", MARKER));

        println!("found marker at {:#x}", base);

        let len = file[search_start..]
            .chunks_exact(marker_size)
            .skip(base / marker_size)
            .position(|val| val != &MARKER_BYTES)
            .map(|count| count * marker_size)
            .expect("marker end not found?");

        println!("marker section is {:#x} bytes long", len);

        if len >= MIN_MARKER_BYTES {
            // big enough contiguous section, should be the actual marker
            break (base + search_start, len);
        }

        search_start += base + len;
        println!(
            "section is not long enough, continuing search from offset {:#x}",
            search_start
        );
    };

    println!("going to patch in {:#x} bytes", to_patch.len());
    assert!(
        to_patch.len() <= len,
        "marker section is not big enough, update kernel::panic::PACKED_SYMBOLS_COUNT"
    );

    let dst = &mut file[base..base + to_patch.len()];
    dst.copy_from_slice(&to_patch);

    println!("nulling out the rest of the markers");
    let dst = &mut file[base + to_patch.len()..base + len];
    dst.fill(0);

    (base, len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_marker() {
        // 6 markers (24 bytes) at offset 12
        let m = MARKER_BYTES;
        let mut buf = vec![
            0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, m[0], m[1], m[2], m[3], m[0], m[1], m[2], m[3],
            m[0], m[1], m[2], m[3], m[0], m[1], m[2], m[3], m[0], m[1], m[2], m[3], m[0], m[1],
            m[2], m[3], 4, 2, 3, 2, 1, 2, 3, 4, 1,
        ];

        let patch = [1; 16];

        let (start, length) = patch_marker_range(&mut buf, &patch);
        assert_eq!(start, 12);
        assert_eq!(length, 24);

        assert_eq!(
            buf,
            vec![
                0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, /* begin patched */
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, /* begin nulled out */
                0, 0, 0, 0, 0, 0, 0, 0, /* begin untouched trailing data */
                4, 2, 3, 2, 1, 2, 3, 4, 1,
            ]
        )
    }

    #[test]
    fn decoys() {
        let m = MARKER_BYTES;
        let mut buf = vec![
            m[0], m[1], m[2], m[3], // oh look the marker
            0, 0, 0, 0, // just kidding
            m[0], m[1], m[2], m[3], // here it is really
            0, 0, 0, 0, // NOPE
            m[0], m[1], m[2], m[3], // surely this time
            m[0], m[1], m[2], m[3], // yes!
            m[0], m[1], m[2], m[3], m[0], m[1], m[2], m[3], 1, 2, 3, 4,
        ];

        let patch = [1; 16];

        let (start, length) = patch_marker_range(&mut buf, &patch);
        assert_eq!(start, 16);
        assert_eq!(length, 16);

        assert_eq!(
            buf,
            vec![
                m[0], m[1], m[2], m[3], 0, 0, 0, 0, m[0], m[1], m[2], m[3], 0, 0, 0, 0,
                // start patch
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // end patch
                1, 2, 3, 4
            ]
        );
    }
}
