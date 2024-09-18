use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct SearchController {
    move_overhead: Duration,
    max_time: Duration,
    max_depth: Depth,
    max_num_nodes_searched: usize,
    max_abs_score_reached: Score,
    stop_search_at_every_node: bool,
    is_infinite_search: bool,
    moves_to_search: Option<Vec<Move>>,
}

impl SearchController {
    #[inline]
    pub fn new() -> Self {
        Self {
            move_overhead: TIMECAT_DEFAULTS.move_overhead,
            max_time: Duration::MAX,
            max_depth: Depth::MAX,
            max_num_nodes_searched: usize::MAX,
            max_abs_score_reached: Score::MAX,
            stop_search_at_every_node: false,
            is_infinite_search: false,
            moves_to_search: None,
        }
    }

    #[inline]
    pub fn is_infinite_search(&self) -> bool {
        self.is_infinite_search
    }

    #[inline]
    pub fn reset_start_time(&mut self) {
        self.stop_search_at_every_node = false;
    }

    pub fn set_max_time(&mut self, duration: Duration) {
        self.max_time = duration;
        self.stop_search_at_every_node = false;
    }

    #[inline]
    pub fn max_time(&self) -> Duration {
        self.max_time
    }

    pub fn is_time_up(&mut self, time_elapsed: Duration) -> bool {
        if self.max_time == Duration::MAX {
            return false;
        }
        self.stop_search_at_every_node = time_elapsed + self.move_overhead >= self.max_time;
        self.stop_search_at_every_node
    }
}

impl<P: PositionEvaluation> SearchControl<Searcher<P>> for SearchController {
    #[inline]
    fn get_move_overhead(&self) -> Duration {
        self.move_overhead
    }

    #[inline]
    fn set_move_overhead(&mut self, duration: Duration) {
        self.move_overhead = duration;
    }

    fn reset_variables(&mut self) {
        self.max_time = Duration::MAX;
        self.max_depth = Depth::MAX;
        self.max_num_nodes_searched = usize::MAX;
        self.max_abs_score_reached = Score::MAX;
        self.stop_search_at_every_node = false;
    }

    fn on_each_search_completion(&mut self, searcher: &mut Searcher<P>) {
        if !self.is_infinite_search()
            && searcher.is_main_threaded()
            && !searcher.is_outside_aspiration_window()
            && searcher.get_depth_completed() >= 10
            && searcher.get_score() >= WINNING_SCORE_THRESHOLD
            && searcher.get_time_elapsed() > Duration::from_secs(10)
        {
            self.stop_search_at_every_node = true;
        }
    }

    fn on_receiving_search_config(&mut self, config: &SearchConfig, searcher: &mut Searcher<P>) {
        self.is_infinite_search = false;
        self.moves_to_search = config.get_moves_to_search().map(|slice| {
            slice
                .iter()
                .copied()
                .filter(|&move_| searcher.get_board().is_legal(move_))
                .collect_vec()
        });
        match config.get_go_command() {
            GoCommand::Ponder => (),
            GoCommand::Infinite => self.is_infinite_search = true,
            GoCommand::Limit {
                depth,
                nodes,
                mate,
                movetime,
                time_clock,
            } => {
                if let Some(depth) = depth {
                    self.max_depth = *depth;
                }
                if let Some(nodes) = nodes {
                    self.max_num_nodes_searched = *nodes;
                }
                if let Some(mate) = mate {
                    self.max_abs_score_reached = searcher
                        .get_evaluator_mut()
                        .evaluate_checkmate_in(2 * *mate);
                }
                if let Some(movetime) = movetime {
                    self.set_max_time(self.max_time.min(*movetime));
                }
                if let Some(TimedGoCommand {
                    wtime,
                    btime,
                    winc,
                    binc,
                    moves_to_go,
                }) = time_clock
                {
                    let board = searcher.get_board();
                    let (self_time, self_inc, opponent_time, _) = match board.turn() {
                        White => (*wtime, *winc, *btime, *binc),
                        Black => (*btime, *binc, *wtime, *winc),
                    };
                    let divider = moves_to_go.unwrap_or(
                        (20 as NumMoves)
                            .checked_sub(board.get_fullmove_number() / 2)
                            .unwrap_or_default()
                            .max(5),
                    );
                    let new_inc = self_inc
                        .checked_sub(Duration::from_secs(1))
                        .unwrap_or_default();
                    let self_time_advantage_bonus =
                        self_time.checked_sub(opponent_time).unwrap_or_default();
                    let opponent_time_advantage =
                        opponent_time.checked_sub(self_time).unwrap_or_default();
                    let mut search_time = self_time
                        .checked_sub(opponent_time_advantage)
                        .unwrap_or_default()
                        / divider as u32
                        + new_inc
                        + self_time_advantage_bonus
                            .checked_sub(Duration::from_secs(10))
                            .unwrap_or_default()
                            / 4;
                    search_time = search_time
                        .max((self_time / 2).min(Duration::from_secs(3)))
                        .min(Duration::from_secs(board.get_fullmove_number() as u64) / 2)
                        .max(Duration::from_millis(100));
                    self.set_max_time(self.max_time.min(search_time));
                }
            }
        }
    }

    #[inline]
    fn get_root_moves_to_search(&self) -> Option<&[Move]> {
        self.moves_to_search.as_deref()
    }

    fn stop_search_at_root_node(&mut self, searcher: &mut Searcher<P>) -> bool {
        searcher.get_depth_completed() >= self.max_depth
            || searcher.get_score().abs() > self.max_abs_score_reached
            || self.stop_search_at_every_node(searcher)
    }

    fn stop_search_at_every_node(&mut self, searcher: &mut Searcher<P>) -> bool {
        if !searcher.is_main_threaded() {
            return false;
        }
        if self.stop_search_at_every_node {
            return true;
        }
        searcher.get_num_nodes_searched() >= self.max_num_nodes_searched
            || self.is_time_up(searcher.get_time_elapsed())
    }
}

impl Default for SearchController {
    fn default() -> Self {
        Self::new()
    }
}
