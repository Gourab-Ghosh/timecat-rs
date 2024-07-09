use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct SearchController {
    move_overhead: Duration,
    max_time: Duration,
    stop_search: bool,
}

impl SearchController {
    pub fn new() -> Self {
        Self {
            move_overhead: TIMECAT_DEFAULTS.move_overhead,
            max_time: Duration::MAX,
            stop_search: false,
        }
    }

    pub fn reset_start_time(&mut self) {
        self.stop_search = false;
    }

    pub fn set_max_time(&mut self, duration: Duration) {
        self.max_time = duration;
        self.stop_search = false;
    }

    pub fn max_time(&self) -> Duration {
        self.max_time
    }

    pub fn is_time_up(&mut self, time_elapsed: Duration) -> bool {
        if self.max_time == Duration::MAX {
            return false;
        }
        self.stop_search = time_elapsed + self.move_overhead >= self.max_time;
        self.stop_search
    }
}

impl SearchControl for SearchController {
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
        self.stop_search = false;
    }

    fn update_control_logic(&mut self, searcher: &Searcher) {
        if self.max_time != Duration::MAX
            && searcher.get_current_depth() >= 10
            && searcher.get_score() >= WINNING_SCORE_THRESHOLD
            && searcher.get_time_elapsed() > Duration::from_secs(10)
        {
            self.stop_search = true;
        }
    }

    fn parse_time_based_go_command(&mut self, searcher: &Searcher, command: GoCommand) {
        let board = searcher.get_board();
        match command {
            GoCommand::MoveTime(duration) => self.set_max_time(duration),
            GoCommand::Timed {
                wtime,
                btime,
                winc,
                binc,
                moves_to_go,
            } => {
                let (self_time, self_inc, opponent_time, _) = match board.turn() {
                    White => (wtime, winc, btime, binc),
                    Black => (btime, binc, wtime, winc),
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
                self.set_max_time(search_time);
            }
            _ => panic!("Expected Timed Command!"),
        }
    }

    fn stop_search(&mut self, searcher: &Searcher) -> bool {
        if !searcher.is_main_threaded() {
            return false;
        }
        if self.stop_search {
            return true;
        }
        self.is_time_up(searcher.get_time_elapsed())
    }
}

impl Default for SearchController {
    fn default() -> Self {
        Self::new()
    }
}
