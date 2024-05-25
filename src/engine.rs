use super::*;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[derive(Clone)]
pub struct GoResponse {
    search_info: SearchInfo,
}

impl GoResponse {
    fn new(search_info: SearchInfo) -> Self {
        Self { search_info }
    }

    pub fn search_info(&self) -> &SearchInfo {
        &self.search_info
    }

    pub fn get_pv(&self) -> &[Option<Move>] {
        self.search_info.get_pv()
    }

    pub fn get_nth_pv_move(&self, n: usize) -> Option<Move> {
        self.search_info.get_pv().get(n).copied().flatten()
    }

    pub fn get_best_move(&self) -> Option<Move> {
        self.get_nth_pv_move(0)
    }

    pub fn get_ponder_move(&self) -> Option<Move> {
        self.get_nth_pv_move(1)
    }

    pub fn get_score(&self) -> Score {
        self.search_info.get_score()
    }
}

pub struct Engine {
    board: Board,
    num_nodes_searched: Arc<AtomicUsize>,
    selective_depth: Arc<AtomicUsize>,
    stopper: Arc<AtomicBool>,
}

impl Engine {
    pub fn new(board: Board) -> Self {
        Self {
            board,
            num_nodes_searched: Arc::new(AtomicUsize::new(0)),
            selective_depth: Arc::new(AtomicUsize::new(0)),
            stopper: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_board(&self) -> &Board {
        &self.board
    }

    pub fn get_board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    fn reset_variables(&self) {
        self.num_nodes_searched.store(0, MEMORY_ORDERING);
        self.selective_depth.store(0, MEMORY_ORDERING);
        self.stopper.store(false, MEMORY_ORDERING);
        TRANSPOSITION_TABLE.reset_variables();
        EVALUATOR.reset_variables();
        if CLEAR_TABLE_AFTER_EACH_SEARCH {
            TRANSPOSITION_TABLE.clear()
        }
    }

    pub fn set_fen(&mut self, fen: &str) -> Result<(), EngineError> {
        let result = self.board.set_fen(fen);
        self.reset_variables();
        result
    }

    pub fn from_fen(fen: &str) -> Result<Self, EngineError> {
        Ok(Engine::new(Board::from_fen(fen)?))
    }

    pub fn get_num_nodes_searched(&self) -> usize {
        self.num_nodes_searched.load(MEMORY_ORDERING)
    }

    pub fn get_selective_depth(&self) -> Ply {
        self.selective_depth.load(MEMORY_ORDERING)
    }

    pub fn generate_searcher(&self, id: usize) -> Searcher {
        Searcher::new(
            id,
            self.board.clone(),
            self.num_nodes_searched.clone(),
            self.selective_depth.clone(),
            self.stopper.clone(),
        )
    }

    fn update_stop_command_from_input(stopper: &AtomicBool) {
        while !stopper.load(MEMORY_ORDERING) {
            match IO_READER
                .read_line_once()
                .unwrap_or_default()
                .to_lowercase()
                .trim()
            {
                "stop" => stopper.store(true, MEMORY_ORDERING),
                "quit" | "exit" => {
                    stopper.store(true, MEMORY_ORDERING);
                    UCI_STATE.set_engine_termination(true);
                }
                _ => {}
            }
        }
    }

    pub fn go(&self, command: GoCommand, print_info: bool) -> GoResponse {
        self.reset_variables();
        let num_threads = UCI_STATE.get_num_threads().max(1);
        let mut join_handles = vec![];
        for id in 1..num_threads {
            let join_handle = thread::spawn({
                let mut threaded_searcher = self.generate_searcher(id);
                move || threaded_searcher.go(GoCommand::Infinite, false)
            });
            join_handles.push(join_handle);
        }
        join_handles.push(thread::spawn({
            let stopper = self.stopper.clone();
            move || Self::update_stop_command_from_input(&stopper)
        }));
        let mut main_thread_searcher = self.generate_searcher(0);
        main_thread_searcher.go(command, print_info);
        self.stopper.store(true, MEMORY_ORDERING);
        for join_handle in join_handles {
            join_handle.join().unwrap();
        }
        let mut search_info = main_thread_searcher.get_search_info();
        if search_info.get_pv().is_empty() && self.board.status() == BoardStatus::Ongoing {
            search_info.set_pv(&[self.board.generate_legal_moves().next()]);
        }
        GoResponse::new(search_info)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(Board::default())
    }
}
