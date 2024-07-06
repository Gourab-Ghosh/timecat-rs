use super::*;

#[derive(Clone, Debug)]
pub struct Timer {
    start_instant: Instant,
    move_overhead: Duration,
    max_time: Duration,
    stop_search: bool,
    stopper: Arc<AtomicBool>,
    is_dummy: bool,
}

impl Timer {
    pub fn new(stopper: Arc<AtomicBool>) -> Self {
        Self {
            start_instant: Instant::now(),
            move_overhead: Duration::ZERO,
            max_time: Duration::MAX,
            stop_search: false,
            stopper,
            is_dummy: false,
        }
    }

    pub fn new_dummy(stopper: Arc<AtomicBool>) -> Self {
        timer::Timer {
            is_dummy: true,
            stopper,
            ..Default::default()
        }
    }

    pub fn reset_variables(&mut self) {
        self.start_instant = Instant::now();
        self.max_time = Duration::MAX;
        self.stop_search = false;
    }

    pub fn reset_start_time(&mut self) {
        self.start_instant = Instant::now();
        self.stop_search = false;
    }

    pub fn set_max_time(&mut self, duration: Duration) {
        self.max_time = duration;
        self.stop_search = false;
    }

    pub fn get_clock(&self) -> Instant {
        self.start_instant
    }

    pub fn time_elapsed(&self) -> Duration {
        self.start_instant.elapsed()
    }

    #[inline]
    pub fn get_move_overhead(&self) -> Duration {
        self.move_overhead
    }

    #[inline]
    pub fn set_move_overhead(&mut self, duration: Duration) {
        self.move_overhead = duration;
    }

    #[inline]
    pub fn with_move_overhead(mut self, duration: Duration) -> Self {
        self.set_move_overhead(duration);
        self
    }

    pub fn max_time(&self) -> Duration {
        self.max_time
    }

    pub fn is_time_up(&mut self) -> bool {
        if self.max_time == Duration::MAX {
            return false;
        }
        self.stop_search = self.time_elapsed() + self.move_overhead >= self.max_time;
        self.stop_search
    }

    pub fn check_stop(&mut self, enable_timer: bool) -> bool {
        if self.stopper.load(MEMORY_ORDERING) {
            return true;
        }
        if self.is_dummy || !enable_timer {
            return false;
        }
        if self.stop_search {
            return true;
        }
        self.is_time_up()
    }

    pub fn update_max_time(&mut self, depth: Depth, score: Score) {
        if self.max_time != Duration::MAX
            && depth >= 10
            && score >= WINNING_SCORE_THRESHOLD
            && self.time_elapsed() > Duration::from_secs(10)
        {
            self.stop_search = true;
        }
    }

    #[cfg(feature = "engine")]
    pub fn parse_time_based_command(&mut self, board: &Board, command: GoCommand) {
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
}

impl Default for Timer {
    fn default() -> Self {
        Self::new(Arc::new(AtomicBool::new(false)))
    }
}
