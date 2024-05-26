This module defines the `Board` structure and its associated methods for managing the game board in the this library. It handles tasks such as initializing the board, displaying the board state, etc.

## Examples
```rust
use timecat::prelude::*;

let board = Board::default();
assert_eq!(board.get_fen(), STARTING_POSITION_FEN)
```

This example initializes a new board.