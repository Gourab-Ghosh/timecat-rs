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
#[derive(Clone, Debug, Default)]
pub struct SearchInfoBuilder {
    sub_board: SubBoard,
    current_depth: Option<Depth>,
    seldepth: Option<Ply>,
    score: Option<Score>,
    nodes: Option<usize>,
    hash_full: Option<f64>,
    overwrites: Option<usize>,
    zero_hit: Option<usize>,
    collisions: Option<usize>,
    time_elapsed: Option<Duration>,
    pv: Vec<Move>,
}

impl SearchInfoBuilder {
    pub fn new(sub_board: SubBoard, pv: Vec<Move>) -> Self {
        Self {
            sub_board,
            pv,
            ..Default::default()
        }
    }

    pub fn set_sub_board(mut self, sub_board: SubBoard) -> Self {
        self.sub_board = sub_board;
        self
    }

    pub fn set_current_depth(mut self, current_depth: Depth) -> Self {
        self.current_depth = Some(current_depth);
        self
    }

    pub fn set_seldepth(mut self, seldepth: Ply) -> Self {
        self.seldepth = Some(seldepth);
        self
    }

    pub fn set_score(mut self, score: Score) -> Self {
        self.score = Some(score);
        self
    }

    pub fn set_nodes(mut self, nodes: usize) -> Self {
        self.nodes = Some(nodes);
        self
    }

    pub fn set_hash_full(mut self, hash_full: f64) -> Self {
        self.hash_full = Some(hash_full);
        self
    }

    pub fn set_overwrites(mut self, overwrites: usize) -> Self {
        self.overwrites = Some(overwrites);
        self
    }

    pub fn set_zero_hit(mut self, zero_hit: usize) -> Self {
        self.zero_hit = Some(zero_hit);
        self
    }

    pub fn set_collisions(mut self, collisions: usize) -> Self {
        self.collisions = Some(collisions);
        self
    }

    pub fn set_time_elapsed(mut self, time_elapsed: Duration) -> Self {
        self.time_elapsed = Some(time_elapsed);
        self
    }

    pub fn set_pv(mut self, pv: Vec<Move>) -> Self {
        self.pv = pv;
        self
    }

    pub fn build(self) -> SearchInfo {
        SearchInfo {
            sub_board: self.sub_board,
            current_depth: self.current_depth,
            seldepth: self.seldepth,
            score: self.score,
            nodes: self.nodes,
            hash_full: self.hash_full,
            overwrites: self.overwrites,
            zero_hit: self.zero_hit,
            collisions: self.collisions,
            time_elapsed: self.time_elapsed,
            pv: self.pv,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct SearchInfo {
    sub_board: SubBoard,
    current_depth: Option<Depth>,
    seldepth: Option<Ply>,
    score: Option<Score>,
    nodes: Option<usize>,
    hash_full: Option<f64>,
    overwrites: Option<usize>,
    zero_hit: Option<usize>,
    collisions: Option<usize>,
    time_elapsed: Option<Duration>,
    pv: Vec<Move>,
}

impl SearchInfo {
    pub fn new(
        sub_board: SubBoard,
        current_depth: Option<Depth>,
        seldepth: Option<Ply>,
        score: Option<Score>,
        nodes: Option<usize>,
        hash_full: Option<f64>,
        overwrites: Option<usize>,
        zero_hit: Option<usize>,
        collisions: Option<usize>,
        time_elapsed: Option<Duration>,
        pv: Vec<Move>,
    ) -> Self {
        Self {
            sub_board,
            current_depth,
            seldepth,
            score,
            nodes,
            hash_full,
            overwrites,
            collisions,
            zero_hit,
            time_elapsed,
            pv,
        }
    }

    #[inline]
    pub fn get_current_depth(&self) -> Option<Depth> {
        self.current_depth
    }

    #[inline]
    pub fn get_num_nodes_searched(&self) -> Option<usize> {
        self.nodes
    }

    #[inline]
    pub fn get_nps(&self) -> Option<u128> {
        Some((self.nodes? as u128 * 10_u128.pow(9)) / self.get_time_elapsed()?.as_nanos())
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
    pub fn get_score(&self) -> Option<Score> {
        self.score
    }

    #[inline]
    pub fn get_score_flipped(&self) -> Option<Score> {
        Some(self.sub_board.score_flipped(self.get_score()?))
    }

    #[inline]
    pub fn get_time_elapsed(&self) -> Option<Duration> {
        self.time_elapsed
    }

    #[inline]
    fn format_info<T: fmt::Display>(desc: &str, info: Option<T>) -> Option<String> {
        let info = info?;
        Some(format!(
            "{} {}",
            desc.trim()
                .trim_end_matches(':')
                .colorize(SUCCESS_MESSAGE_STYLE),
            info,
        ))
    }

    pub fn print_info(&self) {
        let hashfull_string = self.hash_full.map(|hash_full| {
            if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
                format!("{:.2}%", hash_full)
            } else {
                (hash_full.round() as u8).to_string()
            }
        });
        let outputs = [
            Some("info".colorize(INFO_MESSAGE_STYLE)),
            Self::format_info("depth", self.current_depth),
            Self::format_info("seldepth", self.seldepth),
            Self::format_info("score", self.get_score().map(|score| score.stringify())),
            Self::format_info("nodes", self.nodes),
            Self::format_info("nps", self.get_nps()),
            Self::format_info("hashfull", hashfull_string),
            Self::format_info("overwrites", self.overwrites),
            Self::format_info("collisions", self.collisions),
            Self::format_info("zero hit", self.zero_hit),
            Self::format_info(
                "time",
                self.get_time_elapsed().map(|duration| duration.stringify()),
            ),
            Self::format_info("pv", Some(get_pv_string(&self.sub_board, &self.pv))),
        ];
        println_wasm!("{}", outputs.into_iter().flatten().join(" "));
    }

    pub fn print_warning_message(&self, mut alpha: Score, mut beta: Score) {
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            alpha = self.sub_board.score_flipped(alpha);
            beta = self.sub_board.score_flipped(beta);
        }
        let warning_message = format!(
            "info string resetting alpha to -INFINITY and beta to INFINITY at depth {} having alpha {}, beta {} and score {} with time {}",
            if let Some(current_depth) = self.current_depth { current_depth.to_string() } else { "None".to_string() },
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

impl<P: PositionEvaluation> From<&Searcher<P>> for SearchInfo {
    fn from(searcher: &Searcher<P>) -> Self {
        #[cfg(feature = "extras")]
        let (hash_full, overwrites, collisions, zero_hit) = (
            Some(searcher.get_transposition_table().get_hash_full()),
            Some(searcher.get_transposition_table().get_num_overwrites()),
            Some(searcher.get_transposition_table().get_num_collisions()),
            Some(searcher.get_transposition_table().get_zero_hit()),
        );
        #[cfg(not(feature = "extras"))]
        let (hash_full, overwrites, collisions, zero_hit) = (None, None, None, None);
        let mut search_info = Self {
            sub_board: searcher.get_initial_sub_board().to_owned(),
            current_depth: Some(searcher.get_depth_completed().saturating_add(1)),
            seldepth: Some(searcher.get_selective_depth()),
            score: Some(searcher.get_score()),
            nodes: Some(searcher.get_num_nodes_searched()),
            hash_full,
            overwrites,
            collisions,
            zero_hit,
            time_elapsed: Some(searcher.get_time_elapsed()),
            pv: searcher.get_pv().into_iter().copied().collect_vec(),
        };
        search_info.score = search_info
            .score
            .map(|score| search_info.sub_board.score_flipped(score));
        search_info
    }
}
