use std::io::Write;

use sfbinpack::{CompressedTrainingDataEntryReader, CompressedTrainingDataEntryWriter};

fn main() {
    let mut reader = CompressedTrainingDataEntryReader::new(
        "/mnt/g/stockfish-data/test80-2024/test80-2024-06-jun-2tb7p.min-v2.v6.binpack",
    )
    .unwrap();

    let mut count: u64 = 0;

    // let mut writer = CompressedTrainingDataEntryWriter::new(
    //     "/mnt/g/stockfish-data/test80-2024/test80-recreated.binpack",
    //     false,
    // )
    // .unwrap();

    let t0 = std::time::Instant::now();

    while reader.has_next() {
        let entry = reader.next();

        // writer.write_entry(&entry).unwrap();

        count += 1;

        if count % 100000 == 0 {
            let percentage = reader.read_bytes() as f64 / reader.file_size() as f64 * 100.0;

            print_update(count, percentage, t0);
        }
    }

    print!("\x1b[2K");
    print_update(count, 100.0, t0);
    println!();
}

fn print_update(count: u64, percentage: f64, t0: std::time::Instant) {
    let t1 = std::time::Instant::now();
    let elapsed = t1.duration_since(t0).as_millis() + 1;

    print!(
        "count: {} elapsed: {} progress: {} entries/s: {}\r",
        count,
        elapsed,
        percentage,
        (count * 1000) as u128 / elapsed
    );

    std::io::stdout().flush().unwrap()
}
