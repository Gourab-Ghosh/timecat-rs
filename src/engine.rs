use super::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum GoCommand {
    Infinite,
    MoveTime(Duration),
    Depth(Depth),
    // Nodes(usize),
    // Mate(usize),
    // Ponder,
    // SearchMoves(Vec<Move>),
    Timed {
        wtime: Duration,
        btime: Duration,
        winc: Duration,
        binc: Duration,
        moves_to_go: Option<u32>,
    },
}

impl GoCommand {
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

pub struct Engine {
    pub board: Board,
    transposition_table: Arc<TranspositionTable>,
    timer: Timer,
    num_nodes_searched: Arc<AtomicUsize>,
    selective_depth: Arc<AtomicUsize>,
    pv: [Option<Move>; MAX_PLY],
}

impl Engine {
    pub fn new(board: Board) -> Self {
        Self {
            board,
            transposition_table: Arc::new(TranspositionTable::default()),
            timer: Timer::default(),
            num_nodes_searched: Arc::new(AtomicUsize::new(0)),
            selective_depth: Arc::new(AtomicUsize::new(0)),
            pv: [None; MAX_PLY],
        }
    }

    fn reset_variables(&mut self) {
        self.num_nodes_searched.store(0, MEMORY_ORDER);
        self.selective_depth.store(0, MEMORY_ORDER);
        self.timer.reset_variables();
        self.transposition_table.reset_variables();
        // self.board.get_evaluator().lock().unwrap().reset_variables();
    }

    pub fn set_fen(&mut self, fen: &str) -> Result<(), chess::Error> {
        let result = self.board.set_fen(fen);
        self.reset_variables();
        result
    }

    pub fn from_fen(fen: &str) -> Result<Self, chess::Error> {
        Ok(Engine::new(Board::from_fen(fen)?))
    }

    pub fn get_num_nodes_searched(&self) -> usize {
        self.num_nodes_searched.load(MEMORY_ORDER)
    }

    pub fn get_selective_depth(&self) -> Ply {
        self.selective_depth.load(MEMORY_ORDER)
    }

    pub fn get_hash_full(&self) -> f64 {
        self.transposition_table.get_hash_full()
    }

    pub fn get_num_collisions(&self) -> usize {
        self.transposition_table.get_num_collisions()
    }

    pub fn get_pv(&self) -> Vec<Option<Move>> {
        self.pv
            .iter()
            .map(|&m| m)
            .take_while(|&m| m.is_some())
            .collect_vec()
    }

    pub fn get_nth_pv_move(&self, n: usize) -> Option<Move> {
        self.get_pv().get(n).copied().flatten()
    }

    pub fn get_best_move(&self) -> Option<Move> {
        self.get_nth_pv_move(0)
    }

    pub fn get_ponder_move(&self) -> Option<Move> {
        self.get_nth_pv_move(1)
    }

    pub fn generate_searcher(&self, id: usize) -> Searcher {
        Searcher::new(
            id,
            self.board.clone(),
            self.transposition_table.clone(),
            self.timer.clone(),
            self.num_nodes_searched.clone(),
            self.selective_depth.clone(),
        )
    }

    pub fn go(&mut self, command: GoCommand, print_info: bool) -> (Option<Move>, Score) {
        self.reset_variables();
        let num_threads = get_num_threads().max(1);
        for id in 1..num_threads {
            thread::spawn({
                let mut threaded_searcher = self.generate_searcher(id);
                move || threaded_searcher.go(GoCommand::Infinite, false)
            });
        }
        let mut main_thread_searcher = self.generate_searcher(0);
        main_thread_searcher.go(command, print_info);
        let main_thread_searcher_ply_moves = main_thread_searcher.get_pv();
        for ply in 0..MAX_PLY {
            self.pv[ply] = main_thread_searcher_ply_moves.get(ply).copied().flatten();
        }
        (
            self.get_best_move(),
            self.board.score_flipped(main_thread_searcher.get_score()),
        )
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(Board::default())
    }
}
