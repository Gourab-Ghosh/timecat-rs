use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum GoCommand {
    Infinite,
    MoveTime(Duration),
    Depth(Depth),
    // Nodes(usize),
    // Mate(usize),
    Ponder,
    // SearchMoves(Vec<Move>),
    Timed {
        wtime: Duration,
        btime: Duration,
        winc: Duration,
        binc: Duration,
        moves_to_go: Option<NumMoves>,
    },
}

impl GoCommand {
    pub const fn from_millis(millis: u64) -> Self {
        Self::MoveTime(Duration::from_millis(millis))
    }

    pub fn is_infinite(&self) -> bool {
        self == &Self::Infinite
    }

    pub fn is_move_time(&self) -> bool {
        matches!(self, Self::MoveTime(_))
    }

    pub fn is_depth(&self) -> bool {
        matches!(self, Self::Depth(_))
    }

    pub fn is_timed(&self) -> bool {
        matches!(self, Self::Timed { .. })
    }

    pub fn depth_or(&self, depth: Depth) -> Depth {
        match self {
            Self::Depth(depth) => *depth,
            _ => depth,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct SearchInfo {
    sub_board: SubBoard,
    current_depth: Depth,
    seldepth: Ply,
    score: Score,
    nodes: usize,
    #[cfg(feature = "extras")]
    hash_full: f64,
    #[cfg(feature = "extras")]
    overwrites: usize,
    #[cfg(feature = "extras")]
    zero_hit: usize,
    #[cfg(feature = "extras")]
    collisions: usize,
    time_elapsed: Duration,
    pv: Vec<Move>,
}

impl SearchInfo {
    pub fn new(
        sub_board: SubBoard,
        current_depth: Depth,
        seldepth: Ply,
        score: Score,
        nodes: usize,
        #[cfg(feature = "extras")] hash_full: f64,
        #[cfg(feature = "extras")] overwrites: usize,
        #[cfg(feature = "extras")] zero_hit: usize,
        #[cfg(feature = "extras")] collisions: usize,
        time_elapsed: Duration,
        pv: Vec<Move>,
    ) -> Self {
        Self {
            sub_board,
            current_depth,
            seldepth,
            score,
            nodes,
            #[cfg(feature = "extras")]
            hash_full,
            #[cfg(feature = "extras")]
            overwrites,
            #[cfg(feature = "extras")]
            collisions,
            #[cfg(feature = "extras")]
            zero_hit,
            time_elapsed,
            pv,
        }
    }

    #[inline]
    pub fn get_current_depth(&self) -> Depth {
        self.current_depth
    }

    #[inline]
    pub fn get_pv(&self) -> &[Move] {
        self.pv.as_slice()
    }

    #[inline]
    pub fn get_nth_pv_move(&self, n: usize) -> Option<Move> {
        self.get_pv().get(n).copied()
    }

    #[inline]
    pub fn get_best_move(&self) -> Option<Move> {
        self.get_nth_pv_move(0)
    }

    #[inline]
    pub fn get_ponder_move(&self) -> Option<Move> {
        self.get_nth_pv_move(1)
    }

    #[inline]
    pub fn set_pv(&mut self, pv: &[Move]) {
        self.pv = pv.to_vec();
    }

    #[inline]
    pub fn get_score(&self) -> Score {
        self.score
    }

    #[inline]
    pub fn get_score_flipped(&self) -> Score {
        self.sub_board.score_flipped(self.get_score())
    }

    #[inline]
    pub fn get_time_elapsed(&self) -> Duration {
        self.time_elapsed
    }

    #[inline]
    pub fn format_info<T: fmt::Display>(desc: &str, info: T) -> String {
        format!(
            "{} {info}",
            desc.trim()
                .trim_end_matches(':')
                .colorize(SUCCESS_MESSAGE_STYLE)
        )
    }

    pub fn print_info(&self) {
        #[cfg(feature = "extras")]
        let hashfull_string = if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            format!("{:.2}%", self.hash_full)
        } else {
            (self.hash_full.round() as u8).to_string()
        };
        let nps = (self.nodes as u128 * 10_u128.pow(9)) / self.get_time_elapsed().as_nanos();
        let outputs = [
            "info".colorize(INFO_MESSAGE_STYLE),
            Self::format_info("depth", self.current_depth),
            Self::format_info("seldepth", self.seldepth),
            Self::format_info("score", self.get_score().stringify()),
            Self::format_info("nodes", self.nodes),
            Self::format_info("nps", nps),
            #[cfg(feature = "extras")]
            Self::format_info("hashfull", hashfull_string),
            #[cfg(feature = "extras")]
            Self::format_info("overwrites", self.overwrites),
            #[cfg(feature = "extras")]
            Self::format_info("collisions", self.collisions),
            #[cfg(feature = "extras")]
            Self::format_info("zero hit", self.zero_hit),
            Self::format_info("time", self.get_time_elapsed().stringify()),
            Self::format_info("pv", get_pv_string(&self.sub_board, &self.pv)),
        ];
        println_wasm!("{}", outputs.join(" "));
    }

    pub fn print_warning_message(&self, mut alpha: Score, mut beta: Score) {
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            alpha = self.sub_board.score_flipped(alpha);
            beta = self.sub_board.score_flipped(beta);
        }
        let warning_message = format!(
            "info string resetting alpha to -INFINITY and beta to INFINITY at depth {} having alpha {}, beta {} and score {} with time {}",
            self.current_depth,
            alpha.stringify(),
            beta.stringify(),
                if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
                    self.get_score()
                } else {
                    self.get_score_flipped()
                }.stringify(),
            self.get_time_elapsed().stringify(),
        );
        println_wasm!("{}", warning_message.colorize(WARNING_MESSAGE_STYLE));
    }
}

#[cfg(feature = "inbuilt_engine")]
impl<P: PositionEvaluation> From<&Searcher<P>> for SearchInfo {
    fn from(searcher: &Searcher<P>) -> Self {
        let mut search_info = Self {
            sub_board: searcher.get_initial_sub_board().to_owned(),
            current_depth: searcher.get_depth_completed().saturating_add(1),
            seldepth: searcher.get_selective_depth(),
            score: searcher.get_score(),
            nodes: searcher.get_num_nodes_searched(),
            #[cfg(feature = "extras")]
            hash_full: searcher.get_transposition_table().get_hash_full(),
            #[cfg(feature = "extras")]
            overwrites: searcher.get_transposition_table().get_num_overwrites(),
            #[cfg(feature = "extras")]
            collisions: searcher.get_transposition_table().get_num_collisions(),
            #[cfg(feature = "extras")]
            zero_hit: searcher.get_transposition_table().get_zero_hit(),
            time_elapsed: searcher.get_time_elapsed(),
            pv: searcher.get_pv().into_iter().copied().collect_vec(),
        };
        search_info.score = search_info.sub_board.score_flipped(search_info.score);
        search_info
    }
}
