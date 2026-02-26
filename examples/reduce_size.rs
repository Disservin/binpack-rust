use std::fs::OpenOptions;
use std::path::PathBuf;

use sfbinpack::{CompressedTrainingDataEntryReader, CompressedTrainingDataEntryWriter};

fn parse_size(s: &str) -> u64 {
    let s = s.to_lowercase();
    let suffixes = [("gb", 1u64 << 30), ("mb", 1u64 << 20), ("kb", 1u64 << 10)];
    for (suffix, mult) in &suffixes {
        if let Some(num) = s.strip_suffix(suffix) {
            let n: f64 = num.trim().parse().expect("invalid size number");
            return (n * *mult as f64) as u64;
        }
    }
    s.parse()
        .expect("invalid size (use bytes, kb, mb, gb, or %)")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input.binpack> <output.binpack> <size>", args[0]);
        eprintln!("  size examples: 50%, 1gb, 500mb, 100kb, 1073741824");
        std::process::exit(1);
    }

    let input = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);
    let size_arg = &args[3];

    let input_size = std::fs::metadata(&input)
        .expect("cannot stat input file")
        .len();

    let target_bytes: u64 = if let Some(pct) = size_arg.strip_suffix('%') {
        let pct: f64 = pct.parse().expect("invalid percentage");
        ((input_size as f64) * pct / 100.0) as u64
    } else {
        parse_size(size_arg)
    };

    let in_file = OpenOptions::new()
        .read(true)
        .open(&input)
        .expect("cannot open input");

    let out_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&output)
        .expect("cannot open output");

    let mut reader = CompressedTrainingDataEntryReader::new(in_file).unwrap();
    let mut writer = CompressedTrainingDataEntryWriter::new(out_file).unwrap();

    let mut written = 0u64;

    while reader.has_next() {
        let entry = reader.next();
        writer.write_entry(&entry).unwrap();
        written += 1;

        if written % 1000 == 0 {
            let out_size = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
            if out_size >= target_bytes {
                break;
            }
            println!(
                "Written {} entries, output size: {} bytes (target: {} bytes)",
                written, out_size, target_bytes
            );
        }

        reader.read_bytes();
    }

    let final_size = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
    eprintln!(
        "Done. Wrote {} entries, output size: {} bytes (target: {} bytes)",
        written, final_size, target_bytes
    );
}
