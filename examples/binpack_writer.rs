use std::fs::OpenOptions;

use sfbinpack::{
    chess::{
        coords::Square,
        piece::Piece,
        position::Position,
        r#move::{Move, MoveType},
    },
    CompressedTrainingDataEntryWriter, TrainingDataEntry,
};

fn main() {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(false)
        .open("mynew.binpack")
        .unwrap();

    let mut writer = CompressedTrainingDataEntryWriter::new(file).unwrap();

    // When writing a binpack entries must preferably be a contiuation of the previous entry
    // to achieve the best compression ratio.
    let entries = vec![
        TrainingDataEntry {
            pos: Position::from_fen("1q5b/1r5k/4p2p/1b2P1pN/3p4/6PP/1nP3B1/1Q2B1K1 w - - 0 35")
                .unwrap(),
            mv: Move::new(
                Square::new(10),
                Square::new(26),
                MoveType::Normal,
                Piece::none(),
            ),
            score: -201,
            ply: 68,
            result: 0,
        },
        TrainingDataEntry {
            pos: Position::from_fen("1q5b/1r5k/4p2p/1b2P1pN/2Pp4/6PP/1n4B1/1Q2B1K1 b - - 0 35")
                .unwrap(),
            mv: Move::new(
                Square::new(27),
                Square::new(19),
                MoveType::Normal,
                Piece::none(),
            ),
            score: 254,
            ply: 69,
            result: 0,
        },
        TrainingDataEntry {
            pos: Position::from_fen("1q5b/1r5k/4p2p/1b2P1pN/2P5/3p2PP/1n4B1/1Q2B1K1 w - - 0 36")
                .unwrap(),
            mv: Move::new(
                Square::new(14),
                Square::new(49),
                MoveType::Normal,
                Piece::none(),
            ),
            score: -220,
            ply: 70,
            result: 0,
        },
    ];

    for entry in entries.iter() {
        writer.write_entry(entry).unwrap();
    }

    // The writer must be either manually flushed (not advised) or be dropped to flush any remaining data.
}
