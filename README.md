# Timecat Chess Engine

Timecat is a UCI-compatible chess engine designed in Rust that combines powerful algorithms and advanced evaluation techniques to deliver top-notch chess analysis and gameplay. Using alpha-beta pruning with the negamax algorithm and the NNUE evaluation method, Timecat achieves enhanced depth and accuracy in game analysis.

## Timecat as a Library
Timecat was originally conceived as a personal project. However, with the onset of a new chess-related project, I realized the benefits of publishing Timecat as a library to avoid excessive code duplication. Initially designed for personal use, Timecat will now be refined and updated to enhance its functionality and usability as a library, making it more accessible and beneficial for other users. Also the documentation will be further improved.

## Key Features
- **UCI Compatibility:** Fully compatible with the Universal Chess Interface (UCI) standard.
- **Advanced Algorithms:** Utilizes alpha-beta pruning and the negamax algorithm for efficient move searching.
- **NNUE Evaluation:** Incorporates NNUE (efficiently updatable neural network) for state-of-the-art position evaluation.
- **Customizable Builds:** Supports tailored builds through configurable cargo features.

## Integration of the Chess Library
Initially, Timecat was dependent on the external `chess` library, which is available at <https://github.com/jordanbray/chess>. To align more closely with specific requirements, the library was integrated directly into Timecat. This integration permitted significant modifications and extensions to its functionalities, thereby enhancing the engine's overall capabilities. Such integration demonstrates a commitment to adapting and evolving the tools to secure the best possible performance and accuracy in chess analytics.

## User Controls
In the library, only `pub` or non-`pub` visibility modifiers are used (unless extremely necessary to prevent users from making catastrophic errors). This approach ensures that all potentially useful functions and structures are accessible to the user, avoiding the situation where a `pub(crate)` might restrict access to valuable components—a problem I've encountered while using the `chess` library. Therefore, only the features that is considered essential are included in `timecat::prelude`; all other functionalities are available for direct import from the `timecat` library.

Also several cargo features have been introduced to provide users with complete control over the code's behavior.

## NNUE Support
Timecat currently utilizes the Stockfish NNUE for evaluation (only `HalfKP` supported). Plans are in place to transition to a custom-trained NNUE in the future.

## Engine Strength
Although it hasn't been thoroughly tested yet, but my chess engine is capable of defeating [chess.com's max bot](https://www.chess.com/play/computer/Komodo25), which has an Elo rating of 3200.

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

### Compilation with Docker
Clone the repository and compile with native optimizations:
```bash
git clone https://github.com/Gourab-Ghosh/timecat-rs.git
cd timecat-rs
sudo docker build -t timecat .
sudo docker run -it --rm timecat
```

## Usage as a Library

### Minimal Dependency Integration
Integrate Timecat into your Rust projects with minimal dependencies:
```bash
cargo add timecat --no-default-features
```

### Examples
This example demonstrates how to set up a chess board, make moves, evaluate board positions, and utilize the inbuilt engine to find optimal moves using the `timecat` library. Some features such as position evaluation (`nnue`) and engine computation (`engine`) are optional and can be enabled via cargo features.

First, add the timecat crate to your project with the necessary features enabled (`nnue` feature is already enabled within the `engine` feature):
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
    let mut engine = Engine::from_board(board);

    // Configure the engine to search for the best move up to a depth of 10 plies.
    let response = engine.go_verbose(GoCommand::Depth(10));
    let best_move = response.get_best_move()
                            .expect("No best move found");

    // Output the best move found by the engine.
    println!("\nBest Move: {}", best_move);
}
```

You can use UCI commands, although it's not recommended in production environments due to potential parsing delays and unpredictable outputs. The `nnue` and `engine` features are also required in this context.

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
    let mut runner = timecat::TimecatBuilder::<timecat::SearchController>::default().build();

    // List of UCI commands to be executed on the chess engine.
    let uci_commands = [
        // Checks if the engine is ready to receive commands.
        "isready",
        // Sets the move overhead option.
        "setoption name move overhead value 200",
        // Display the current state of the chess board.
        "d",
        // Sets a new game position by applying the moves.
        "position startpos moves e2e4 e7e5",
        // Instructs the engine to calculate the best move within 3000 milliseconds.
        "go movetime 3000",
    ];

    // Process each UCI command and handle potential errors.
    for command in uci_commands {
        runner.run_uci_command(command)?;
    }

    Ok(())
}
```

Or just enjoy the engine play against itself:
```rust
use timecat::prelude::*;
use std::time::Duration;
use std::error::Error;

fn main() {
    self_play(
        &mut Engine::default(),
        GoCommand::MoveTime(Duration::from_millis(10)),
        // set to verbose mode (true/false)
        true,
        // Limit to number of moves to play (u16/Some(u16)/None), None denoting no limit
        100,
    );
}
```

The `selfplay` command works on the binary as well.

## Cargo Features
- `binary`: Enables binary builds, including NNUE and engine functionalities.
- `nnue_reader`: Adds support for NNUE evaluation by reading nnue files.
- `inbuilt_nnue`: Integrate built-in NNUE evaluation support by including the nnue file directly into the binary, fetched using the minreq library.
- `engine`: Provides the Engine struct for in-depth position analysis and move searching.
- `non_binary_feature`: Adds some functionalities not needed in binary, to get better insights of the behavior of the code.
- `colored`: Displays all information in a visually appealing colored format for enhanced readability.
- `speed`: Optimize the code to improve speed by using unsafe rust. Note that the gain in speed might not be significant in some cases.
- `serde`: Enables serialization and deserialization support via `serde`.
- `wasm`: Webassembly support (Still in Testing phase).
- `debug`: Intended solely for development use.
- `experimental`: Codes under development for upcoming features.

Default features include `binary`, `colored` and `speed`.

## TODO
- [ ] Implement other variants of chess.
- [ ] Implement Syzygy Tablebase.
- [ ] Organize the Polyglot Table codes to make it usable.
- [ ] Organize the pgn related codes to make it usable.
- [ ] Implement xboard feature.
- [ ] Add svg feature like the python package chess for better visualization.

## License
Timecat is open-sourced under the [GNU GENERAL PUBLIC LICENSE](https://github.com/Gourab-Ghosh/timecat-rs/blob/master/LICENSE). You are free to use, modify, and distribute it under the same license.

## Contributing
Feel free to fork the repository, make improvements, and submit pull requests. You can also report issues or suggest features through the GitHub issue tracker.