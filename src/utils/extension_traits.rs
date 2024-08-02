use super::*;

pub trait Compress {
    type CompressedItem;

    fn compress(self) -> Self::CompressedItem;
}

pub trait Decompress<T> {
    fn decompress(self) -> T;
}

#[cfg(feature = "colored")]
pub trait CustomColorize {
    fn colorize(&self, style_functions: &[ColoredStringFunction]) -> String;
}

#[cfg(not(feature = "colored"))]
pub trait CustomColorize {
    fn colorize(&self, _: &[fn(String) -> String]) -> String;
}

#[cfg(feature = "nnue_reader")]
pub trait ClippedRelu<InputType, OutputType, const N: usize> {
    fn clipped_relu(
        &self,
        scale_by_pow_of_two: OutputType,
        min: InputType,
        max: InputType,
    ) -> MathVec<OutputType, N>;

    fn clipped_relu_into(
        &self,
        scale_by_pow_of_two: OutputType,
        min: InputType,
        max: InputType,
        output: &mut [OutputType; N],
    );
}

pub trait Stringify {
    fn stringify(&self) -> String;
}

pub trait StringifyScore {
    fn stringify_score(self) -> String;
    fn stringify_score_uci(self) -> String;
}

pub trait StringifyMove {
    fn uci(self) -> String;
    fn algebraic(self, sub_board: &SubBoard, long: bool) -> Result<String>;
    fn stringify_move(self, sub_board: &SubBoard) -> Result<String>;

    fn san(self, sub_board: &SubBoard) -> Result<String>
    where
        Self: Sized,
    {
        self.algebraic(sub_board, false)
    }

    fn lan(self, sub_board: &SubBoard) -> Result<String>
    where
        Self: Sized,
    {
        self.algebraic(sub_board, true)
    }
}

// TODO: Try to remove static lifetime from the trait
pub trait SearchControl<Searcher>: Clone + Send + 'static {
    fn get_move_overhead(&self) -> Duration;
    fn set_move_overhead(&mut self, duration: Duration);
    fn reset_variables(&mut self);
    fn stop_search_at_root_node(&mut self, searcher: &Searcher) -> bool;
    fn stop_search_at_every_node(&mut self, searcher: &Searcher) -> bool;
    fn on_receiving_go_command(&mut self, command: GoCommand, searcher: &Searcher);
    fn on_each_search_completion(&mut self, searcher: &Searcher);

    #[inline]
    fn with_move_overhead(mut self, duration: Duration) -> Self {
        self.set_move_overhead(duration);
        self
    }
}

// TODO: Try to remove static lifetime from the trait
pub trait PositionEvaluation: Clone + Send + 'static {
    fn evaluate(&mut self, sub_board: &SubBoard) -> Score;

    #[inline]
    fn reset_variables(&mut self) {}

    #[inline]
    fn clear(&mut self) {}

    #[inline]
    fn print_info(&self) {}

    #[inline]
    fn evaluate_flipped(&mut self, sub_board: &SubBoard) -> Score {
        sub_board.score_flipped(self.evaluate(sub_board))
    }

    #[inline]
    fn evaluate_checkmate(&mut self, mate_distance: usize) -> Score {
        CHECKMATE_SCORE - mate_distance as Score
    }

    #[inline]
    fn evaluate_draw(&mut self) -> Score {
        0
    }
}

pub trait ChessEngine {
    type IoReader;

    fn get_board(&self) -> &Board;
    fn get_board_mut(&mut self) -> &mut Board;
    fn set_transposition_table_size(&self, size: CacheTableSize);
    fn set_num_threads(&mut self, num_threads: NonZeroUsize);
    fn set_move_overhead(&mut self, duration: Duration);
    fn get_num_nodes_searched(&self) -> usize;
    fn terminate(&self) -> bool;
    fn set_termination(&self, b: bool);
    fn clear_hash(&mut self);
    fn evaluate_current_position(&mut self) -> Score;
    fn evaluate_current_position_flipped(&mut self) -> Score;
    fn go(&mut self, command: GoCommand, verbose: bool) -> SearchInfo;

    fn set_fen(&mut self, fen: &str) -> Result<()> {
        self.get_board_mut().set_fen(fen)?;
        self.reset_variables();
        Ok(())
    }

    #[inline]
    fn print_info(&self) {}

    #[inline]
    fn reset_variables(&mut self) {}

    #[inline]
    #[allow(unused_variables)]
    fn set_optional_io_reader(&mut self, optional_io_reader: Self::IoReader) {}

    #[inline]
    #[must_use = "If you don't need the response, you can just search the position."]
    fn go_quiet(&mut self, command: GoCommand) -> SearchInfo {
        self.go(command, false)
    }

    #[inline]
    #[must_use = "If you don't need the response, you can just search the position."]
    fn go_verbose(&mut self, command: GoCommand) -> SearchInfo {
        self.go(command, true)
    }

    fn with_io_reader(mut self, optional_io_reader: Self::IoReader) -> Self
    where
        Self: Sized,
    {
        self.set_optional_io_reader(optional_io_reader);
        self
    }
}

pub trait SubBoardMethodOverload<T> {
    fn parse_san(&self, _: &str) -> Result<T>;
    fn parse_lan(&self, _: &str) -> Result<T>;
    fn parse_uci(&self, _: &str) -> Result<T>;
    fn make_move_new(&self, _: T) -> Self;
    #[inline]
    fn make_move(&mut self, valid_or_null_move: T)
    where
        Self: Sized,
    {
        *self = self.make_move_new(valid_or_null_move);
    }
}

pub trait BoardMethodOverload<T> {
    // TODO: Avoid Code Repetition
    fn push_unchecked(&mut self, _: T);
    fn push(&mut self, _: T) -> Result<()>;
    fn gives_repetition(&self, _: T) -> bool;
    fn gives_threefold_repetition(&self, _: T) -> bool;
    fn gives_claimable_threefold_repetition(&self, _: T) -> bool;
}

pub trait SearcherMethodOverload<T> {
    fn push_unchecked(&mut self, _: T);
}
