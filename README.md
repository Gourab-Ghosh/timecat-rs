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
This snippet below demonstrate setting up a chess board, making moves, evaluating positions, and utilizing the engine to determine the best moves. Note that some features, such as position evaluation and engine utilization, require enabling specific cargo features (`nnue` and `engine`).
```rust
use timecat::prelude::*;

fn main() {
    // Generate a board with initial position.
    let mut board = Board::default();

    // Make moves
    board.push_san("e4").unwrap();
    board.push_san("e5").unwrap();

    // Print current evaluation (Requires nnue feature)
    println!("Current Evaluation: {}", board.evaluate());

    // Create an engine (Requires engine feature)
    let engine = Engine::new(board);

    // Search best move
    let response = engine.go_verbose(GoCommand::Depth(10));
    let best_move = response.get_best_move().unwrap();

    println!("{}", best_move);
}
```

## Cargo Features
- `default`: Activates all standard features.
- `binary`: Enables binary builds, including NNUE and engine functionalities.
- `nnue`: Adds support for NNUE (downloaded via `reqwest`).
- `engine`: Provides the Engine struct for in-depth position analysis and move searching.
- `serde`: Enables serialization and deserialization support via `serde`.

## License
Timecat is open-sourced under the [GNU GENERAL PUBLIC LICENSE](LICENSE). You are free to use, modify, and distribute it under the same license.

## Contributing
We welcome contributions! Feel free to fork the repository, make improvements, and submit pull requests. You can also report issues or suggest features through the GitHub issue tracker.