# Stockfish Binpack

Rust port of the Stockfish binpack reader from the [C++ version](https://github.com/official-stockfish/Stockfish/blob/tools/src/extra/nnue_data_binpack_format.h).

Binpacks store chess positions and their evaluations in a compact format. Instead of storing complete positions, they store the differences between moves. This makes them very space efficient - using only 2.5 bytes per position on average.

## Compile

If your machine has the fast BMI2 instruction set (Zen 3+), you should enable the feature flag

```bash
cargo build --release --features bmi2;
```

or define it in your `Cargo.toml` file (change version).

```
[dependencies]
binpack = { version = "0.1.0", features = ["bmi2"] }
```

## Usage

Run the following Cargo command in your project directory:

```shell
cargo add sfbinpack
```

```rust
use sfbinpack::CompressedTrainingDataEntryReader;

fn main() {
    let mut reader = CompressedTrainingDataEntryReader::new(
        "test80.binpack", // path to file
    )
    .unwrap();

    while reader.has_next() {
        let entry = reader.next();

        println!("entry:");
        println!("fen {}", entry.pos.fen());
        println!("uci {:?}", entry.mv.as_uci());
        println!("score {}", entry.score);
        println!("ply {}", entry.ply);
        println!("result {}", entry.result);
        println!("\n");

        // progress percentage
        // let percentage = reader.read_bytes() as f64 / reader.file_size() as f64 * 100.0;
    }
}
```

_More examples can be found in the [examples](./examples) directory._  
_If you are doing some counting keep in mind to use a `u64` type for the counter._

## Compression

When compressing new data, it is advised to store the entire continuation of the actual game.
This will allow for a much better compression ratio.  
Failure to do so will result in a larger file size, than compared to other alternatives.

## Performance Comparison

Slightly faster when compiled with bmi2 because of _pdep_u64 trick which is missing in the upstream version.

## License

GNU General Public License v3.0

<https://www.gnu.org/licenses/gpl-3.0.html>
