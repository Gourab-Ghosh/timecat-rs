This module defines the `Board` structure and its associated methods for managing the game board in the this library. It handles tasks such as initializing the board, displaying the board state, etc.

## Structures
- **Board**
  - Represents the game board with methods to manipulate and query its state.

## Useful Methods
### Board
- `new() -> Board`
  - Creates a new board with the default starting fen `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`.

## Examples
```rust
use timecat::prelude::*;

let board = Board::default();
assert_eq!(board.get_fen(), STARTING_POSITION_FEN)
```

This example initializes a new board.

## Errors
- **To be updated soon**
  - To be updated soon.