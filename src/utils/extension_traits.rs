use super::*;

pub trait UniqueIdentifier {
    fn unique_identifier(&self) -> impl PartialEq + Hash;
}

pub trait Compress {
    type CompressedItem;

    fn compress(self) -> Self::CompressedItem;
}

pub trait Decompress<T> {
    fn decompress(self) -> Result<T>;
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

pub trait StringifyScore {
    fn stringify_score_console<'a>(self) -> Cow<'a, str>;
    fn stringify_score_uci<'a>(self) -> Cow<'a, str>;
    fn stringify_score<'a>(self) -> Cow<'a, str>;
}

pub trait StringifyMove {
    fn uci<'a>(self) -> Cow<'a, str>;
    fn algebraic<'a>(self, position: &ChessPosition, long: bool) -> Result<Cow<'a, str>>;
    fn stringify_move<'a>(self, position: &ChessPosition) -> Result<Cow<'a, str>>;

    fn san<'a>(self, position: &ChessPosition) -> Result<Cow<'a, str>>
    where
        Self: Sized,
    {
        self.algebraic(position, false)
    }

    fn lan<'a>(self, position: &ChessPosition) -> Result<Cow<'a, str>>
    where
        Self: Sized,
    {
        self.algebraic(position, true)
    }
}

pub trait StringifyHash {
    fn stringify_hash(&self) -> String;
}

pub trait Stringify {
    fn stringify<'a>(&self) -> Cow<'a, str>;
}

// TODO: Try to remove static lifetime from the trait
pub trait SearchControl<Searcher>: Clone + Send + 'static {
    fn get_move_overhead(&self) -> Duration;
    fn set_move_overhead(&mut self, duration: Duration);
    fn reset_variables(&mut self);
    fn stop_search_at_root_node(&mut self, searcher: &mut Searcher) -> bool;
    fn stop_search_at_every_node(&mut self, searcher: &mut Searcher) -> bool;
    fn on_receiving_search_config(&mut self, config: &SearchConfig, searcher: &mut Searcher);
    fn on_each_search_completion(&mut self, searcher: &mut Searcher);

    #[inline]
    fn with_move_overhead(mut self, duration: Duration) -> Self {
        self.set_move_overhead(duration);
        self
    }

    #[inline]
    fn get_root_moves_to_search(&self) -> Option<&[Move]> {
        None
    }
}

// TODO: Try to remove static lifetime from the trait
pub trait PositionEvaluation: Clone + Send + 'static {
    fn evaluate(&mut self, position: &ChessPosition) -> Score;

    #[inline]
    fn reset_variables(&mut self) {}

    #[inline]
    fn clear(&mut self) {}

    #[inline]
    fn print_info(&self) {}

    #[inline]
    fn evaluate_flipped(&mut self, position: &ChessPosition) -> Score {
        position.score_flipped(self.evaluate(position))
    }

    #[inline]
    fn evaluate_checkmate_in(&mut self, mate_distance: Ply) -> Score {
        if CHECKMATE_SCORE as Ply > mate_distance {
            CHECKMATE_SCORE - mate_distance as Score
        } else {
            0
        }
    }

    #[inline]
    fn evaluate_checkmated_in(&mut self, mate_distance: Ply) -> Score {
        -self.evaluate_checkmate_in(mate_distance)
    }

    #[inline]
    fn evaluate_draw(&mut self) -> Score {
        0
    }
}

macro_rules! generate_chess_engine_methods {
    ($func_name:ident, ( $( $parameter_name:ident: $parameter_type:ty ),+ $(,)? ) , $input:expr $(,)?) => {
        #[inline]
        #[must_use = "If you don't need the search info, you can just search the position."]
        fn $func_name(&mut self, $($parameter_name: $parameter_type),+, verbose: bool) -> SearchInfo {
            self.search($input, verbose)
        }
        generate_chess_engine_methods!(@quiet_and_verbose $func_name, ( $($parameter_name: $parameter_type),+ ));
    };

    (@quiet_and_verbose $func_name:ident, ( $( $parameter_name:ident: $parameter_type:ty ),+ $(,)? ) $(,)?) => {
        paste::item! {
            #[inline]
            #[must_use = "If you don't need the search info, you can just search the position."]
            fn [<$func_name _quiet>](&mut self, $($parameter_name: $parameter_type),+) -> SearchInfo {
                self.[<$func_name>]( $($parameter_name),+, false)
            }

            #[inline]
            #[must_use = "If you don't need the search info, you can just search the position."]
            fn [<$func_name _verbose>](&mut self, $($parameter_name: $parameter_type),+) -> SearchInfo {
                self.[<$func_name>]( $($parameter_name),+, true)
            }
        }
    };
}

pub trait ChessEngine {
    type IoReader;

    fn get_board(&self) -> &Board;
    fn get_board_mut(&mut self) -> &mut Board;
    fn set_transposition_table_size(&self, size: CacheTableSize);
    fn set_num_threads(&mut self, num_threads: NonZeroUsize);
    fn set_move_overhead(&mut self, duration: Duration);
    fn get_opening_book(&self) -> Option<&dyn PolyglotBook>;
    fn set_opening_book<B: PolyglotBook + 'static>(&mut self, book: Option<Arc<B>>);
    fn terminate(&self) -> bool;
    fn set_termination(&self, b: bool);
    fn set_fen(&mut self, fen: &str) -> Result<()>;
    fn clear_hash(&mut self);
    fn evaluate_current_position(&mut self) -> Score;
    fn evaluate_current_position_flipped(&mut self) -> Score;
    #[must_use = "If you don't need the search info, you can just search the position."]
    fn search(&mut self, config: &SearchConfig, verbose: bool) -> SearchInfo;

    #[inline]
    fn print_info(&self) {}

    #[inline]
    #[expect(unused_variables)]
    fn set_optional_io_reader(&mut self, optional_io_reader: Self::IoReader) {}

    #[inline]
    fn get_opening_book_weighted_move(&self) -> Option<WeightedMove> {
        self.get_opening_book()?
            .get_best_weighted_move(self.get_board())
            .filter(|WeightedMove { move_, .. }| self.get_board().is_legal(move_))
    }

    fn with_io_reader(mut self, optional_io_reader: Self::IoReader) -> Self
    where
        Self: Sized,
    {
        self.set_optional_io_reader(optional_io_reader);
        self
    }

    generate_chess_engine_methods!(@quiet_and_verbose search, (config: &SearchConfig));
    generate_chess_engine_methods!(search_depth, (depth: Depth), &GoCommand::from_depth(depth).into());
    generate_chess_engine_methods!(search_nodes, (nodes: usize), &GoCommand::from_nodes(nodes).into());
    generate_chess_engine_methods!(search_mate, (mate: Ply), &GoCommand::from_mate(mate).into());
    generate_chess_engine_methods!(search_movetime, (movetime: Duration), &GoCommand::from_movetime(movetime).into());
    generate_chess_engine_methods!(search_millis, (millis: u64), &GoCommand::from_millis(millis).into());
    generate_chess_engine_methods!(
        search_timed,
        (
            wtime: Duration,
            btime: Duration,
            winc: Duration,
            binc: Duration,
            moves_to_go: impl Into<Option<NumMoves>>,
        ),
        &GoCommand::from_timed(
            wtime,
            btime,
            winc,
            binc,
            moves_to_go.into(),
        ).into(),
    );
    generate_chess_engine_methods!(go, (go_command: GoCommand), &go_command.into());
}

pub trait BoardPositionMethodOverload<T> {
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
    unsafe fn push_unchecked(&mut self, _: T);
    fn push(&mut self, _: T) -> Result<()>;
    fn gives_repetition(&self, _: T) -> bool;
    fn gives_threefold_repetition(&self, _: T) -> bool;
    fn gives_claimable_threefold_repetition(&self, _: T) -> bool;
}

pub trait PolyglotBook {
    fn read_from_path(book_path: &str) -> Result<Self>
    where
        Self: Sized;
    fn get_best_weighted_move(&self, board: &Board) -> Option<WeightedMove>;
}

pub trait SearcherMethodOverload<T> {
    unsafe fn push_unchecked(&mut self, _: T);
}

#[cfg(feature = "serde")]
pub trait SerdeSerialize {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>;
}

#[cfg(feature = "serde")]
pub trait SerdeDeserialize<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        Self: Sized;
}

pub trait FloatExtensions {
    #[allow(clippy::wrong_self_convention)]
    fn is_integer(self) -> bool;
}

macro_rules! impl_float {
    ($($float: ty),+ $(,)?) => {
        $(
            impl FloatExtensions for $float {
                #[inline]
                fn is_integer(self) -> bool {
                    self.trunc().to_bits() == self.to_bits()
                }
            }
        )+
    };
}

impl_float!(f32, f64);
