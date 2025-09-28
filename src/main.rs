use std::{
    fs::{File, OpenOptions},
    io::Write,
};

use sfbinpack::CompressedTrainingDataEntryReader;

fn main() {
    let file =
        File::open("..\\..\\stockfish-data\\test80-2024-06-jun-2tb7p.min-v2.v6.binpack").unwrap();

    let filesize = file.metadata().unwrap().len();

    let mut reader = CompressedTrainingDataEntryReader::new(file).unwrap();

    let mut num_entries: u64 = 0;

    // let mut writer = CompressedTrainingDataEntryWriter::new(
    //     "/mnt/g/stockfish-data/test80-2024/test80-recreated.binpack",
    //     false,
    // )
    // .unwrap();

    let t0 = std::time::Instant::now();

    while reader.has_next() {
        let entry = reader.next();

        // Check if the next entry is a continuation of the current entry
        // reader.is_next_entry_continuation();

        num_entries += 1;

        if num_entries.is_multiple_of(1000000) {
            let percentage = reader.read_bytes() as f64 / filesize as f64 * 100.0;

            print_update(num_entries, percentage, t0);
        }
    }

    print!("\x1b[2K");
    print_update(num_entries, 100.0, t0);
    println!();
}

fn print_update(num_entries: u64, percentage: f64, t0: std::time::Instant) {
    let t1 = std::time::Instant::now();
    let elapsed = t1.duration_since(t0).as_secs().max(1) as f64;
    let entries_per_second = num_entries as f64 / elapsed;

    print!(
        "count: {} elapsed: {:.2}s progress: {:.2}% entries/s: {:.2}\r",
        num_entries, elapsed, percentage, entries_per_second
    );

    std::io::stdout().flush().unwrap()
}
