# Stockfish Binpack

Rust port of the Stockfish binpack reader from the [C++ version](https://github.com/official-stockfish/Stockfish/blob/tools/src/extra/nnue_data_binpack_format.h).

Binpacks store chess positions and their evaluations in a compact format.
Instead of storing complete positions, they store the differences between moves.
This makes them very space efficient - using only 2.5 bytes per position on
average. See [Anatomy](#anatomy) for more details.

## Compile

If your machine has the fast BMI2 instruction set (Zen 3+), you should enable the feature flag

```bash
cargo build --release --features bmi2;
```

or define it in your `Cargo.toml` file (change version).

```
[dependencies]
binpack = { version = "0.4.3", features = ["bmi2"] }
```

## Usage

Run the following Cargo command in your project directory:

```shell
cargo add sfbinpack
```

```rust
use sfbinpack::CompressedTrainingDataEntryReader;

fn main() {
    let file = File::open("data.binpack").unwrap();
    let mut reader = CompressedTrainingDataEntryReader::new(file).unwrap();

    while reader.has_next() {
        let entry = reader.next();

        println!("entry:");
        println!("fen {}", entry.pos.fen().unwrap());
        println!("uci {:?}", entry.mv.as_uci());
        println!("score {}", entry.score);
        println!("ply {}", entry.ply);
        println!("result {}", entry.result);
        println!("\n");
    }
}
```

_More examples can be found in the [examples](./examples) directory._  
_If you are doing some counting keep in mind to use a `u64` type for the counter._

## Examples

To run the examples in the `examples` directory, use the following command:

```shell
cargo run --release --example <example_name>
```

`binpack_reader` - Read a binpack file and print the contents.
`binpack_writer` - Write a binpack file from a list of positions.

## Performance Comparison

Slightly faster when compiled with bmi2 because of _pdep_u64 trick which is missing in the upstream version.

## Anatomy

![Binpack](./img/binpack2x.png)

<!-- ## EBNF -->

<!-- The extended Backus-Naur form (EBNF) of the binpack format is as follows: -->

<!-- ```
(* BINP Format EBNF Specification *)
File = { Block } ;
Block = ChunkHeader , { Chain } ;
ChunkHeader = Magic , ChunkSize ;
Magic = '"BINP"' ;
ChunkSize = UINT32LE ;  (* 4 bytes, little endian *)
Chain = Stem , Count , MoveText ;
Stem = Position , Move , Score , PlyResult , Rule50 ;
Count = UINT16BE ;  (* 2 bytes, big endian *)
MoveText = { MoveScore } ;

(* Stem components - total 32 bytes )
Position = CompressedPosition ;  ( 24 bytes *)
Move = CompressedMove ;  (* 2 bytes *)
Score = INT16BE ;  (* 2 bytes, big endian, signed *)
PlyResult = UINT8 ;  (* 2 byte, big endian unsigned *)
Rule50 = UINT16BE ;  (* 2 bytes, big endian *)

(* MoveText components *)
MoveScore = EncodedMove , EncodedScore ;

(* Encoded components )
EncodedMove = VARLEN_UINT ;  ( Variable length encoding *)
EncodedScore = VARLEN_INT ;  (* Variable length encoding *)

(* Terminal symbols *)
UINT32LE = ? 4-byte unsigned integer in little-endian format ? ;
UINT16BE = ? 2-byte unsigned integer in big-endian format ? ;
INT16BE = ? 2-byte signed integer in big-endian format ? ;
UINT8 = ? 1-byte unsigned integer ? ;
VARLEN_UINT = ? Variable-length encoded unsigned integer ? ;
VARLEN_INT = ? Variable-length encoded signed integer ? ;
CompressedPosition = ? 24-byte compressed chess position ? ;
CompressedMove = ? 2-byte compressed chess move ? ;
``` -->

## Compression

When compressing new data, it is advised to store the entire continuation of the actual game.
This will allow for a much better compression ratio.  
Failure to do so will result in a larger file size, than compared to other alternatives.

## License

GNU General Public License v3.0

<https://www.gnu.org/licenses/gpl-3.0.html>
