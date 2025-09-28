use std::fs::OpenOptions;

use sfbinpack::CompressedTrainingDataEntryReader;

fn main() {
    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .append(false)
        .open("./test/ep1.binpack")
        .unwrap();

    let mut reader = CompressedTrainingDataEntryReader::new(file).unwrap();

    while reader.has_next() {
        let entry = reader.next();

        println!("entry:");
        println!("fen {}", entry.pos.fen().unwrap());
        println!("uci move {:?}", entry.mv.as_uci());
        println!("score {}", entry.score);
        println!("ply {}", entry.ply);
        println!("result {}", entry.result);
        println!("\n");
    }
}
