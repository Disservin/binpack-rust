use std::{
    ffi::{CStr, CString},
    fs::OpenOptions,
    os::raw::c_char,
    path::Path,
    ptr,
    sync::{Mutex, OnceLock},
};

use crate::{
    chess::{
        castling_rights::CastleType,
        coords::Square,
        piece::Piece,
        piecetype::PieceType,
        position::Position,
        r#move::{Move, MoveType},
    },
    writer::{CompressedTrainingDataEntryWriter, CompressedWriterError},
    TrainingDataEntry,
};

#[repr(C)]
pub struct SfbinpackEntry {
    pub fen: *const c_char,
    pub uci_move: *const c_char,
    pub score: i16,
    pub ply: u16,
    pub result: i16,
}

#[repr(C)]
pub struct SfbinpackWriterHandle {
    inner: CompressedTrainingDataEntryWriter<std::fs::File>,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfbinpackStatus {
    Ok = 0,
    NullPointer = -1,
    InvalidUtf8 = -2,
    FenParseFailed = -3,
    MoveParseFailed = -4,
    WriterClosed = -5,
    IoError = -6,
}

static LAST_ERROR: OnceLock<Mutex<Option<CString>>> = OnceLock::new();

fn error_slot() -> &'static Mutex<Option<CString>> {
    LAST_ERROR.get_or_init(|| Mutex::new(None))
}

fn set_last_error(message: impl AsRef<str>) {
    let cstring = CString::new(message.as_ref())
        .unwrap_or_else(|_| CString::new("invalid error message").unwrap());
    *error_slot().lock().unwrap() = Some(cstring);
}

fn clear_last_error() {
    *error_slot().lock().unwrap() = None;
}

fn convert_entry(entry: &SfbinpackEntry) -> Result<TrainingDataEntry, SfbinpackStatus> {
    let fen_cstr = unsafe { CStr::from_ptr(entry.fen) };
    let fen = fen_cstr.to_str().map_err(|_| {
        set_last_error("FEN string contained invalid UTF-8");
        SfbinpackStatus::InvalidUtf8
    })?;

    let move_cstr = unsafe { CStr::from_ptr(entry.uci_move) };
    let uci_move = move_cstr.to_str().map_err(|_| {
        set_last_error("UCI move string contained invalid UTF-8");
        SfbinpackStatus::InvalidUtf8
    })?;

    let position = Position::from_fen(fen).map_err(|_| {
        set_last_error("Failed to parse FEN string");
        SfbinpackStatus::FenParseFailed
    })?;

    let mv = parse_uci_move(uci_move, &position).map_err(|msg| {
        set_last_error(msg);
        SfbinpackStatus::MoveParseFailed
    })?;

    Ok(TrainingDataEntry {
        pos: position,
        mv,
        score: entry.score,
        ply: entry.ply,
        result: entry.result,
    })
}

fn parse_uci_move(uci: &str, position: &Position) -> Result<Move, String> {
    if uci.len() < 4 {
        return Err("UCI move must contain at least 4 characters".into());
    }

    let from = Square::from_string(&uci[0..2])
        .ok_or_else(|| "Invalid from-square in UCI move".to_string())?;
    let to = Square::from_string(&uci[2..4])
        .ok_or_else(|| "Invalid to-square in UCI move".to_string())?;

    let piece = position.piece_at(from);
    if piece == Piece::none() {
        return Err("No piece on from-square described in UCI move".into());
    }

    let from_file_idx = (from.index() % 8) as i32;
    let to_file_idx = (to.index() % 8) as i32;

    if piece.piece_type() == PieceType::King && (from_file_idx - to_file_idx).abs() == 2 {
        let castle_type = if to_file_idx > from_file_idx {
            CastleType::Short
        } else {
            CastleType::Long
        };
        return Ok(Move::from_castle(castle_type, position.side_to_move()));
    }

    if uci.len() > 5 {
        return Err("UCI move contains unexpected trailing characters".into());
    }

    let mut move_type = MoveType::Normal;
    let mut promotion_piece = Piece::none();

    if uci.len() == 5 {
        move_type = MoveType::Promotion;
        let promo = uci.chars().nth(4).unwrap();
        let side = position.side_to_move();
        let piece_type = match promo {
            'q' | 'Q' => PieceType::Queen,
            'r' | 'R' => PieceType::Rook,
            'b' | 'B' => PieceType::Bishop,
            'n' | 'N' => PieceType::Knight,
            _ => return Err("Unsupported promotion piece in UCI move".into()),
        };
        promotion_piece = Piece::new(piece_type, side);
    }

    if piece.piece_type() == PieceType::Pawn
        && from_file_idx != to_file_idx
        && position.piece_at(to) == Piece::none()
    {
        move_type = MoveType::EnPassant;
    }

    Ok(match move_type {
        MoveType::Normal => Move::normal(from, to),
        MoveType::Promotion => Move::promotion(from, to, promotion_piece),
        MoveType::EnPassant => Move::en_passant(from, to),
        MoveType::Castle => unreachable!("castling handled earlier"),
    })
}

#[no_mangle]
pub unsafe extern "C" fn sfbinpack_writer_new(path: *const c_char) -> *mut SfbinpackWriterHandle {
    if path.is_null() {
        set_last_error("Path pointer was null");
        return ptr::null_mut();
    }

    let c_path = CStr::from_ptr(path);
    let path_str = match c_path.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Path contained invalid UTF-8");
            return ptr::null_mut();
        }
    };

    let file = match open_file(path_str) {
        Ok(f) => f,
        Err(e) => {
            set_last_error(&format!("Failed to open output file: {}", e));
            return ptr::null_mut();
        }
    };

    match CompressedTrainingDataEntryWriter::new(file) {
        Ok(writer) => {
            clear_last_error();
            Box::into_raw(Box::new(SfbinpackWriterHandle { inner: writer }))
        }
        Err(CompressedWriterError::Io(e)) => {
            set_last_error(&format!("IO error while creating writer: {}", e));
            ptr::null_mut()
        }
        Err(err) => {
            set_last_error(&format!("Failed to create writer: {}", err));
            ptr::null_mut()
        }
    }
}

fn open_file(path_str: &str) -> std::io::Result<std::fs::File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(Path::new(path_str))
}

#[no_mangle]
pub unsafe extern "C" fn sfbinpack_writer_write_entry(
    writer: *mut SfbinpackWriterHandle,
    entry: *const SfbinpackEntry,
) -> SfbinpackStatus {
    if writer.is_null() || entry.is_null() {
        set_last_error("Writer or entry pointer was null");
        return SfbinpackStatus::NullPointer;
    }

    let writer = &mut *writer;
    let training_entry = match convert_entry(&*entry) {
        Ok(entry) => entry,
        Err(status) => return status,
    };

    match writer.inner.write_entry(&training_entry) {
        Ok(_) => {
            clear_last_error();
            SfbinpackStatus::Ok
        }
        Err(CompressedWriterError::Io(e)) => {
            set_last_error(&format!("IO error while writing entry: {}", e));
            SfbinpackStatus::IoError
        }
        Err(err) => {
            set_last_error(&format!("Failed to write entry: {}", err));
            SfbinpackStatus::IoError
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn sfbinpack_writer_flush(
    writer: *mut SfbinpackWriterHandle,
) -> SfbinpackStatus {
    if writer.is_null() {
        set_last_error("Writer pointer was null");
        return SfbinpackStatus::NullPointer;
    }

    let writer = &mut *writer;
    writer.inner.flush();
    clear_last_error();
    SfbinpackStatus::Ok
}

#[no_mangle]
pub unsafe extern "C" fn sfbinpack_writer_finish(
    writer: *mut SfbinpackWriterHandle,
) -> SfbinpackStatus {
    if writer.is_null() {
        set_last_error("Writer pointer was null");
        return SfbinpackStatus::NullPointer;
    }

    let writer = &mut *writer;
    writer.inner.flush_and_end();
    clear_last_error();
    SfbinpackStatus::Ok
}

#[no_mangle]
pub unsafe extern "C" fn sfbinpack_writer_free(writer: *mut SfbinpackWriterHandle) {
    if writer.is_null() {
        return;
    }

    drop(Box::from_raw(writer));
}

#[no_mangle]
pub extern "C" fn sfbinpack_last_error_message() -> *const c_char {
    let guard = error_slot().lock().unwrap();
    guard.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::CompressedTrainingDataEntryReader;
    use std::{ffi::CString, fs::File};
    use tempfile::tempdir;

    #[test]
    fn write_entry_from_c() {
        let tmp_dir = tempdir().unwrap();
        let path = tmp_dir.path().join("ffi.binpack");

        let path_c = CString::new(path.to_str().unwrap()).unwrap();
        let writer_ptr = unsafe { sfbinpack_writer_new(path_c.as_ptr()) };
        assert!(!writer_ptr.is_null(), "writer pointer should not be null");

        let fen = CString::new("rnbqkbnr/ppp2ppp/8/3pp3/3P4/5N2/PPP1PPPP/RNBQKB1R w KQkq e6 0 3")
            .unwrap();
        let mv = CString::new("c1g5").unwrap();

        let entry = SfbinpackEntry {
            fen: fen.as_ptr(),
            uci_move: mv.as_ptr(),
            score: 24,
            ply: 4,
            result: 1,
        };

        let status = unsafe { sfbinpack_writer_write_entry(writer_ptr, &entry) };
        assert_eq!(status, SfbinpackStatus::Ok);

        let status = unsafe { sfbinpack_writer_finish(writer_ptr) };
        assert_eq!(status, SfbinpackStatus::Ok);

        unsafe { sfbinpack_writer_free(writer_ptr) };

        let file = File::open(path).unwrap();
        let mut reader = CompressedTrainingDataEntryReader::new(file).unwrap();
        assert!(reader.has_next());
        let read_entry = reader.next();
        assert_eq!(read_entry.score, 24);
        assert_eq!(read_entry.mv.as_uci(), "c1g5");
        assert_eq!(read_entry.result, 1);
    }
}
