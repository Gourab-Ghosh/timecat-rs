use super::*;

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
    fn get_transposition_table(&self) -> &TranspositionTable {
        &self.transposition_table
    }

    fn reset_variables(&mut self) {
        self.num_nodes_searched.store(0, MEMORY_ORDERING);
        self.selective_depth.store(0, MEMORY_ORDERING);
        self.controller.reset_variables();
        self.board.get_evaluator().reset_variables();
        if CLEAR_TABLE_AFTER_EACH_SEARCH {
            self.transposition_table.clear()
        }
        self.transposition_table.reset_variables();
        self.set_stop_command(false);
        self.set_termination(false);
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

impl<T: SearchControl> ChessEngine for CustomEngine<T> {
    type TranspositionTable = TranspositionTable;
    type IoReader = IoReader;

    #[inline]
    fn get_board(&self) -> &Board {
        &self.board
    }

    #[inline]
    fn get_board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    fn set_fen(&mut self, fen: &str) -> Result<()> {
        self.board.set_fen(fen)?;
        self.reset_variables();
        Ok(())
    }

    #[inline]
    fn evaluate_board(&mut self) -> Score {
        self.board.evaluate()
    }

    fn set_transposition_table_size(&self, size: CacheTableSize) {
        self.transposition_table.set_size(size);
        if GLOBAL_TIMECAT_STATE.is_in_debug_mode() {
            self.transposition_table.print_info();
        }
    }

    #[inline]
    fn get_num_threads(&self) -> usize {
        self.num_threads.get()
    }

    #[inline]
    fn set_num_threads(&mut self, num_threads: NonZeroUsize) {
        self.num_threads = num_threads;
    }

    #[inline]
    fn get_move_overhead(&self) -> Duration {
        self.controller.get_move_overhead()
    }

    #[inline]
    fn set_move_overhead(&mut self, duration: Duration) {
        self.controller.set_move_overhead(duration);
    }

    #[inline]
    fn get_num_nodes_searched(&self) -> usize {
        self.num_nodes_searched.load(MEMORY_ORDERING)
    }

    #[inline]
    fn terminate(&self) -> bool {
        self.terminate.load(MEMORY_ORDERING)
    }

    #[inline]
    fn set_termination(&self, b: bool) {
        self.terminate.store(b, MEMORY_ORDERING);
    }

    fn clear_hash(&self) {
        self.get_transposition_table().clear();
        self.get_board().get_evaluator().clear();
    }

    fn print_info(&self) {
        print_engine_version();
        println_wasm!();
        self.transposition_table.print_info();
        self.board.get_evaluator().print_info();
    }

    #[inline]
    fn get_optional_io_reader(&self) -> Option<Self::IoReader> {
        self.optional_io_reader.clone()
    }

    #[inline]
    fn set_optional_io_reader(&mut self, optional_io_reader: Self::IoReader) {
        self.optional_io_reader = Some(optional_io_reader);
    }

    #[must_use = "If you don't need the response, you can just search the position."]
    fn go(&mut self, command: GoCommand, verbose: bool) -> SearchInfo {
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
        search_info
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
