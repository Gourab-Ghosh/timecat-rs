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
pub struct GoResponse {
    search_info: SearchInfo,
}

impl GoResponse {
    #[inline]
    fn new(search_info: SearchInfo) -> Self {
        Self { search_info }
    }

    #[inline]
    pub fn search_info(&self) -> &SearchInfo {
        &self.search_info
    }

    #[inline]
    pub fn get_pv(&self) -> &[Move] {
        self.search_info.get_pv()
    }

    #[inline]
    pub fn get_nth_pv_move(&self, n: usize) -> Option<Move> {
        self.search_info.get_pv().get(n).copied()
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
    pub fn get_score(&self) -> Score {
        self.search_info.get_score()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct CustomEngine<T: SearchControl> {
    board: Board,
    transposition_table: SerdeWrapper<Arc<TranspositionTable>>,
    controller: T,
    num_threads: NonZeroUsize,
    num_nodes_searched: SerdeWrapper<Arc<AtomicUsize>>,
    selective_depth: SerdeWrapper<Arc<AtomicUsize>>,
    optional_io_reader: Option<IoReader>,
    stop_command: SerdeWrapper<Arc<AtomicBool>>,
    terminate: SerdeWrapper<Arc<AtomicBool>>,
}

impl<T: SearchControl> CustomEngine<T> {
    pub fn new(board: Board, transposition_table: TranspositionTable, controller: T) -> Self {
        Self {
            board,
            transposition_table: transposition_table.into(),
            controller,
            num_threads: TIMECAT_DEFAULTS.num_threads,
            num_nodes_searched: AtomicUsize::new(0).into(),
            selective_depth: AtomicUsize::new(0).into(),
            optional_io_reader: None,
            stop_command: AtomicBool::new(false).into(),
            terminate: AtomicBool::new(false).into(),
        }
    }

    #[inline]
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    #[inline]
    pub fn get_board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    #[inline]
    pub fn get_transposition_table(&self) -> &TranspositionTable {
        &self.transposition_table
    }

    #[inline]
    pub fn get_optional_io_reader(&self) -> Option<IoReader> {
        self.optional_io_reader.clone()
    }

    #[inline]
    pub fn set_optional_io_reader(&mut self, optional_io_reader: IoReader) {
        self.optional_io_reader = Some(optional_io_reader);
    }

    pub fn with_io_reader(mut self, optional_io_reader: IoReader) -> Self {
        self.set_optional_io_reader(optional_io_reader);
        self
    }

    fn reset_variables(&mut self) {
        self.num_nodes_searched.store(0, MEMORY_ORDERING);
        self.selective_depth.store(0, MEMORY_ORDERING);
        self.transposition_table.reset_variables();
        self.controller.reset_variables();
        self.board.get_evaluator().reset_variables();
        if CLEAR_TABLE_AFTER_EACH_SEARCH {
            self.transposition_table.clear()
        }
        self.set_stop_command(false);
        self.set_termination(false);
    }

    pub fn set_fen(&mut self, fen: &str) -> Result<()> {
        let result = self.board.set_fen(fen);
        self.reset_variables();
        result
    }

    #[inline]
    pub fn get_num_threads(&self) -> usize {
        self.num_threads.get()
    }

    #[inline]
    pub fn set_num_threads(&mut self, num_threads: NonZeroUsize) {
        self.num_threads = num_threads;
    }

    #[inline]
    pub fn get_search_controller(&self) -> &impl SearchControl {
        &self.controller
    }

    #[inline]
    pub fn get_search_controller_mut(&mut self) -> &mut impl SearchControl {
        &mut self.controller
    }

    #[inline]
    pub fn get_num_nodes_searched(&self) -> usize {
        self.num_nodes_searched.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn get_selective_depth(&self) -> Ply {
        self.selective_depth.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn generate_searcher(&self, id: usize) -> Searcher {
        Searcher::new(
            id,
            self.board.clone(),
            self.transposition_table.clone().into_inner(),
            self.num_nodes_searched.clone().into_inner(),
            self.selective_depth.clone().into_inner(),
            self.stop_command.clone().into_inner(),
        )
    }

    #[inline]
    pub fn get_stop_command(&self) -> bool {
        self.stop_command.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn set_stop_command(&self, b: bool) {
        self.stop_command.store(b, MEMORY_ORDERING);
    }

    #[inline]
    pub fn terminate(&self) -> bool {
        self.terminate.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn set_termination(&self, b: bool) {
        self.terminate.store(b, MEMORY_ORDERING);
    }

    fn update_stop_command(
        stop_command: Arc<AtomicBool>,
        io_reader: IoReader,
        terminate: Arc<AtomicBool>,
    ) {
        while !stop_command.load(MEMORY_ORDERING) {
            match io_reader
                .read_line_once()
                .unwrap_or_default()
                .to_lowercase()
                .trim()
            {
                "stop" => stop_command.store(true, MEMORY_ORDERING),
                "quit" | "exit" => {
                    stop_command.store(true, MEMORY_ORDERING);
                    terminate.store(true, MEMORY_ORDERING);
                }
                _ => {}
            }
        }
    }
}

impl<T: SearchControl> CustomEngine<T> {
    pub fn go(&mut self, command: GoCommand, verbose: bool) -> GoResponse {
        self.reset_variables();
        let mut join_handles = vec![];
        for id in 1..self.num_threads.get() {
            let mut threaded_searcher = self.generate_searcher(id);
            let controller = self.controller.clone();
            let join_handle = thread::spawn(move || {
                threaded_searcher.go(GoCommand::Infinite, controller, false);
            });
            join_handles.push(join_handle);
        }
        if let Some(io_reader) = self.optional_io_reader.as_ref() {
            let stop_command = self.stop_command.clone().into_inner();
            let reader = io_reader.clone();
            let terminate = self.terminate.clone().into_inner();
            join_handles.push(thread::spawn(move || {
                Self::update_stop_command(stop_command, reader, terminate);
            }));
        }
        let mut main_thread_searcher = self.generate_searcher(0);
        main_thread_searcher.go(command, self.controller.clone(), verbose);
        self.set_stop_command(true);
        for join_handle in join_handles {
            join_handle.join().unwrap();
        }
        let mut search_info = main_thread_searcher.get_search_info();
        if search_info.get_pv().is_empty() && self.board.status() == BoardStatus::Ongoing {
            search_info.set_pv(&[self.board.generate_legal_moves().next().unwrap()]);
        }
        GoResponse::new(search_info)
    }

    #[inline]
    pub fn go_quiet(&mut self, command: GoCommand) -> GoResponse {
        self.go(command, false)
    }

    #[inline]
    pub fn go_verbose(&mut self, command: GoCommand) -> GoResponse {
        self.go(command, true)
    }
}

impl<T: SearchControl + Default> CustomEngine<T> {
    #[inline]
    pub fn from_board(board: Board) -> Self {
        Self::new(board, TranspositionTable::default(), T::default())
    }

    #[inline]
    pub fn from_fen(fen: &str) -> Result<Self> {
        Ok(Self::from_board(Board::from_fen(fen)?))
    }
}

impl<T: SearchControl> Clone for CustomEngine<T> {
    fn clone(&self) -> Self {
        Self {
            board: self.board.clone(),
            transposition_table: self.transposition_table.as_ref().clone().into(),
            controller: self.controller.clone(),
            num_nodes_searched: AtomicUsize::new(self.num_nodes_searched.load(MEMORY_ORDERING))
                .into(),
            selective_depth: AtomicUsize::new(self.selective_depth.load(MEMORY_ORDERING)).into(),
            optional_io_reader: self.optional_io_reader.clone(),
            stop_command: AtomicBool::new(self.stop_command.load(MEMORY_ORDERING)).into(),
            terminate: AtomicBool::new(self.terminate.load(MEMORY_ORDERING)).into(),
            ..*self
        }
    }
}

impl<T: SearchControl + Default> Default for CustomEngine<T> {
    fn default() -> Self {
        Self::new(
            Board::default(),
            TranspositionTable::default(),
            T::default(),
        )
    }
}
