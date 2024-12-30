use std::io::Write;

use sfbinpack::reader::data_reader::CompressedTrainingDataEntryReader;

fn main() {
    let mut reader = CompressedTrainingDataEntryReader::new(
        // "/mnt/g/stockfish-data/test80-2024/test80-2024-06-jun-2tb7p.min-v2.v6.binpack",
        // "/mnt/g/stockfish-data/dual-nnue/hse-v1/leela96-filt-v2.min.high-simple-eval-1k.min-v2.binpack",
        "/mnt/g/stockfish-data/dual-nnue/hse-v1/test60-2019-2tb7p.min.high-simple-eval-1k.min-v2.binpack",
        // "/mnt/g/stockfish-data/ep1.binpack",
    )
    .unwrap();

    let mut count: u64 = 0;

    let t0 = std::time::Instant::now();

    while reader.has_next() {
        let _entry = reader.next();

        count += 1;

        // println!("entry:");
        // println!("{}", entry.pos.fen());
        // println!("{:?}", entry.mv.as_uci());
        // println!("score {}", entry.score);
        // println!("ply {}", entry.ply);
        // println!("result {}", entry.result);
        // println!("\n");

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

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_reader() {
    //     let mut reader = CompressedTrainingDataEntryReader::new(
    //         "/mnt/g/stockfish-data/test80-2024/test80-2024-06-jun-2tb7p.min-v2.v6.binpack",
    //     )
    //     .unwrap();

    //     let mut count: u64 = 0;
    //     let mut score: i64 = 0;

    //     while reader.has_next() {
    //         let entry = reader.next();

    //         count += 1;

    //         score += entry.score as i64;

    //         if count == 100000 {
    //             println!("count: {}", count);
    //             println!("score: {}", score);
    //             break;
    //         }
    //     }
    // }

    #[test]
    fn test_reader_simple() {
        let mut reader = CompressedTrainingDataEntryReader::new("./test/ep1.binpack").unwrap();

        let mut count: u64 = 0;
        let mut score: i64 = 0;

        while reader.has_next() {
            let entry = reader.next();

            count += 1;

            score += entry.score as i64;
        }

        assert_eq!(count, 3);
        assert_eq!(score, -167);
    }
}
