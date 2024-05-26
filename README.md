# Timecat Chess Engine

Timecat is a UCI-compatible chess engine designed in Rust that combines powerful algorithms and advanced evaluation techniques to deliver top-notch chess analysis and gameplay. Using alpha-beta pruning with the negamax algorithm and the NNUE evaluation method, Timecat achieves enhanced depth and accuracy in game analysis.

## Key Features
- **UCI Compatibility:** Fully compatible with the Universal Chess Interface (UCI) standard.
- **Advanced Algorithms:** Utilizes alpha-beta pruning and the negamax algorithm for efficient move searching.
- **NNUE Evaluation:** Incorporates NNUE (efficiently updatable neural network) for state-of-the-art position evaluation.
- **Customizable Builds:** Supports tailored builds through configurable cargo features.

## Integration of the Chess Library
Initially, Timecat was dependent on the external `chess` library, which is available at https://github.com/jordanbray/chess. To align more closely with specific requirements, the library was integrated directly into Timecat. This integration permitted significant modifications and extensions to its functionalities, thereby enhancing the engine's overall capabilities. Such integration demonstrates a commitment to adapting and evolving the tools to secure the best possible performance and accuracy in chess analytics.

## NNUE Support
Timecat currently utilizes the Stockfish NNUE for evaluation. Plans are in place to transition to a custom-trained NNUE in the future.

## Installation

### Installing as a Binary
Optimize your setup for the best performance:
```bash
RUSTFLAGS="-C target-cpu=native" cargo install timecat
```

### Compilation from Source
Clone the repository and compile with native optimizations:
```bash
git clone https://github.com/Gourab-Ghosh/timecat-rs.git
cd timecat-rs
RUSTFLAGS="-C target-cpu=native" cargo run --release
```

## Usage as a Library

### Minimal Dependency Integration
Integrate Timecat into your Rust projects with minimal dependencies:
```bash
cargo add timecat --no-default-features
```

### Examples
This example demonstrates how to set up a chess board, make moves, evaluate board positions, and utilize the inbuilt engine to find optimal moves in Rust using the `timecat` library. Some features such as position evaluation (`nnue`) and engine computation (`engine`) are optional and can be enabled via cargo features.

First, add the timecat crate to your project with the necessary features enabled (`nnue` evaluation feature is already included in the `engine` feature):
```bash
cargo add timecat --no-default-features --features engine
```

Then, you can proceed with the following Rust code:

```rust
use timecat::prelude::*;

fn main() {
    // Initialize a chess board with the default starting position.
    let mut board = Board::default();

    // Apply moves in standard algebraic notation.
    board.push_san("e4").expect("Failed to make move: e4");
    board.push_san("e5").expect("Failed to make move: e5");

    // Evaluate the current board position using the nnue feature.
    let evaluation = board.evaluate();
    println!("Current Evaluation: {}", evaluation);

    // Initialize the chess engine with the current board state.
    let engine = Engine::new(board);

    // Configure the engine to search for the best move up to a depth of 10 plies.
    let response = engine.go_verbose(GoCommand::Depth(10));
    let best_move = response.get_best_move()
                            .expect("No best move found");

    // Output the best move found by the engine.
    println!("Best Move: {}", best_move);
}
```

## Cargo Features
- `binary`: Enables binary builds, including NNUE and engine functionalities.
- `nnue`: Adds support for NNUE (downloaded via `reqwest`).
- `engine`: Provides the Engine struct for in-depth position analysis and move searching.
- `colored_output`: Displays all information in a visually appealing colored format for enhanced readability.
- `serde`: Enables serialization and deserialization support via `serde`.

Default features include `binary` and `colored_output`

## License
Timecat is open-sourced under the [GNU GENERAL PUBLIC LICENSE](LICENSE). You are free to use, modify, and distribute it under the same license.

## Contributing
We welcome contributions! Feel free to fork the repository, make improvements, and submit pull requests. You can also report issues or suggest features through the GitHub issue tracker.