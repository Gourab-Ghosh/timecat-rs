# Timecat Chess Engine

Timecat is a UCI-compatible chess engine designed in Rust that combines powerful algorithms and advanced evaluation techniques to deliver top-notch chess analysis and gameplay. Using alpha-beta pruning with the negamax algorithm and the NNUE evaluation method, Timecat achieves enhanced depth and accuracy in game analysis.

## Key Features
- **UCI Compatibility:** Fully compatible with the Universal Chess Interface (UCI) standard.
- **Advanced Algorithms:** Utilizes alpha-beta pruning and the negamax algorithm for efficient move searching.
- **NNUE Evaluation:** Incorporates NNUE (efficiently updatable neural network) for state-of-the-art position evaluation.
- **Customizable Builds:** Supports tailored builds through configurable cargo features.

## Integration of the Chess Library
Initially, Timecat was dependent on the external `chess` library, which is available at <https://github.com/jordanbray/chess>. To align more closely with specific requirements, the library was integrated directly into Timecat. This integration permitted significant modifications and extensions to its functionalities, thereby enhancing the engine's overall capabilities. Such integration demonstrates a commitment to adapting and evolving the tools to secure the best possible performance and accuracy in chess analytics.

## `pub` vs `pub(crate)`
In the library, we only use `pub` or non-`pub` visibility modifiers. This approach ensures that all potentially useful functions and structures are accessible to the user, avoiding the situation where a `pub(crate)` might restrict access to valuable components—a problem I've encountered while using the `chess` library. Therefore, only the features I consider essential are included in `timecat::prelude`; all other functionalities are available for direct import from the `timecat` library.

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

First, add the timecat crate to your project with the necessary features enabled (`nnue` feature is already included in the `engine` feature):
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
    println!("Current Evaluation: {}\n", evaluation);

    // Initialize the engine with the current board state.
    let engine = Engine::new(board);

    // Configure the engine to search for the best move up to a depth of 10 plies.
    let response = engine.go_verbose(GoCommand::Depth(10));
    let best_move = response.get_best_move()
                            .expect("No best move found");

    // Output the best move found by the engine.
    println!("\nBest Move: {}", best_move);
}
```

You can use UCI commands, although it's not recommended in production environments due to potential parsing delays and unpredictable outputs. The 'nnue' and 'engine' features are also required in this context.

As previous, add the timecat crate to your project:
```bash
cargo add timecat --no-default-features --features engine
```

Then, you can proceed with the following Rust code:
```rust
use timecat::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Create the default engine initialized with the standard starting position.
    let mut engine = Engine::default();

    // Enable UCI (Universal Chess Interface) mode explicitly.
    // Some UCI commands may not work without this.
    timecat::UCI_STATE.set_uci_mode(true, false);

    // List of UCI commands to be executed on the chess engine.
    let uci_commands = [
        // Checks if the engine is ready to receive commands.
        "isready",
        // Sets the move overhead option.
        "setoption name move overhead value 200",
        // Display the current state of the chess board.
        "d",
        // Sets a new game position by applying the moves.
        "position startpos e2e4 e7e5",
        // Instructs the engine to calculate the best move within 3000 milliseconds.
        "go movetime 3000",
    ];

    // Process each UCI command and handle potential errors.
    for command in uci_commands {
        timecat::Parser::parse_command(&mut engine, command)?;
    }

    Ok(())
}
```

> **Caution:** To ensure compatibility with UCI commands, activate UCI mode by using the following code:<br>
> `timecat::UCI_STATE.set_uci_mode(true, false);`<br>
> Failure to do so may result in some UCI commands not functioning as expected.

Or just enjoy the engine play against itself:
```rust
use timecat::prelude::*;
use std::error::Error;

fn main() {
    timecat::Parser::parse_command(
        &mut Engine::default(),
        // selfplay command has same format as go command
        "selfplay movetime 10", // Adjust time according to your wish
    ).unwrap();
}
```

The `selfplay` command works on the binary as well.

## Cargo Features
- `binary`: Enables binary builds, including NNUE and engine functionalities.
- `nnue`: Adds support for NNUE (downloaded via `reqwest`).
- `engine`: Provides the Engine struct for in-depth position analysis and move searching.
- `colored_output`: Displays all information in a visually appealing colored format for enhanced readability.
- `speed`: Optimize the code to improve speed at the cost of increased memory usage and in extremely rare cases cause unpredictable behavior. Note that the gain in speed might be minimal compared to the additional memory required.
- `serde`: Enables serialization and deserialization support via `serde`.

Default features include `binary`, `colored_output` and `speed`.

## TODO
- [ ] Implement other variants of chess.
- [ ] Implement Syzygy Tablebase.
- [ ] Organize the Polyglot Table codes to make it usable.
- [ ] Organize the pgn related codes to make it usable.
- [ ] Implement xboard feature.
- [ ] Add svg feature for support like the python package chess for better visualization.

## License
Timecat is open-sourced under the [GNU GENERAL PUBLIC LICENSE](https://github.com/Gourab-Ghosh/timecat-rs/blob/master/LICENSE). You are free to use, modify, and distribute it under the same license.

## Contributing
We welcome contributions! Feel free to fork the repository, make improvements, and submit pull requests. You can also report issues or suggest features through the GitHub issue tracker.