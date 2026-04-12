#ifndef SFBINPACK_H
#define SFBINPACK_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SfbinpackWriterHandle sfbinpack_writer_handle;

typedef enum SfbinpackStatus {
    SFBINPACK_STATUS_OK = 0,
    SFBINPACK_STATUS_NULL_POINTER = -1,
    SFBINPACK_STATUS_INVALID_UTF8 = -2,
    SFBINPACK_STATUS_FEN_PARSE_FAILED = -3,
    SFBINPACK_STATUS_MOVE_PARSE_FAILED = -4,
    SFBINPACK_STATUS_WRITER_CLOSED = -5,
    SFBINPACK_STATUS_IO_ERROR = -6,
} SfbinpackStatus;

typedef struct SfbinpackEntry {
    const char* fen;
    const char* uci_move;
    short score;
    unsigned short ply;
    short result;
} SfbinpackEntry;

sfbinpack_writer_handle* sfbinpack_writer_new(const char* path);
SfbinpackStatus sfbinpack_writer_write_entry(sfbinpack_writer_handle* writer, const SfbinpackEntry* entry);
SfbinpackStatus sfbinpack_writer_flush(sfbinpack_writer_handle* writer);
SfbinpackStatus sfbinpack_writer_finish(sfbinpack_writer_handle* writer);
void sfbinpack_writer_free(sfbinpack_writer_handle* writer);
const char* sfbinpack_last_error_message(void);

#ifdef __cplusplus
}
#endif

#endif /* SFBINPACK_H */
