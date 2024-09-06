use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct EngineProperties {
    _use_mate_distance_pruning: bool,
    _clear_table_after_each_search: bool,
    _use_lmr: bool,
}

impl EngineProperties {
    pub fn use_mate_distance_pruning(&self) -> bool {
        self._use_mate_distance_pruning
    }

    pub fn set_using_mate_distance_pruning(&mut self, value: bool) {
        self._use_mate_distance_pruning = value;
    }

    pub fn clear_table_after_each_search(&self) -> bool {
        self._clear_table_after_each_search
    }

    pub fn set_clearing_table_after_each_search(&mut self, value: bool) {
        self._clear_table_after_each_search = value;
    }

    pub fn use_lmr(&self) -> bool {
        self._use_lmr
    }

    pub fn set_using_lmr(&mut self, value: bool) {
        self._use_lmr = value;
    }
}

impl Default for EngineProperties {
    fn default() -> Self {
        Self {
            _use_mate_distance_pruning: true,
            _clear_table_after_each_search: true,
            _use_lmr: true,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct CustomEngine<T: SearchControl<Searcher<P>>, P: PositionEvaluation> {
    board: Board,
    transposition_table: SerdeWrapper<Arc<TranspositionTable>>,
    evaluator: P,
    controller: T,
    num_threads: NonZeroUsize,
    num_nodes_searched: SerdeWrapper<Arc<AtomicUsize>>,
    selective_depth: SerdeWrapper<Arc<AtomicUsize>>,
    optional_io_reader: Option<IoReader>,
    stop_command: SerdeWrapper<Arc<AtomicBool>>,
    terminate: SerdeWrapper<Arc<AtomicBool>>,
    properties: EngineProperties,
}

impl<T: SearchControl<Searcher<P>>, P: PositionEvaluation> CustomEngine<T, P> {
    pub fn new(
        board: Board,
        transposition_table: TranspositionTable,
        controller: T,
        evaluator: P,
    ) -> Self {
        Self {
            board,
            transposition_table: transposition_table.into(),
            evaluator,
            controller,
            num_threads: TIMECAT_DEFAULTS.num_threads,
            num_nodes_searched: AtomicUsize::new(0).into(),
            selective_depth: AtomicUsize::new(0).into(),
            optional_io_reader: None,
            stop_command: AtomicBool::new(false).into(),
            terminate: AtomicBool::new(false).into(),
            properties: EngineProperties::default(),
        }
    }

    #[inline]
    fn get_transposition_table(&self) -> &TranspositionTable {
        &self.transposition_table
    }

    #[inline]
    pub fn get_search_controller(&self) -> &impl SearchControl<Searcher<P>> {
        &self.controller
    }

    #[inline]
    pub fn get_search_controller_mut(&mut self) -> &mut impl SearchControl<Searcher<P>> {
        &mut self.controller
    }

    #[inline]
    pub fn get_properties(&self) -> &EngineProperties {
        &self.properties
    }

    #[inline]
    pub fn get_properties_mut(&mut self) -> &mut EngineProperties {
        &mut self.properties
    }

    #[inline]
    fn get_num_nodes_searched(&self) -> usize {
        self.num_nodes_searched.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn get_selective_depth(&self) -> Ply {
        self.selective_depth.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn get_num_threads(&self) -> usize {
        self.num_threads.get()
    }

    #[inline]
    pub fn get_move_overhead(&self) -> Duration {
        self.controller.get_move_overhead()
    }

    #[inline]
    pub fn get_optional_io_reader(&self) -> Option<IoReader> {
        self.optional_io_reader.clone()
    }

    pub fn reset_variables(&mut self) {
        self.num_nodes_searched.store(0, MEMORY_ORDERING);
        self.selective_depth.store(0, MEMORY_ORDERING);
        self.controller.reset_variables();
        self.evaluator.reset_variables();
        if self.properties.clear_table_after_each_search() {
            self.transposition_table.clear();
        }
        self.transposition_table.reset_variables();
        self.set_stop_command(false);
        self.set_termination(false);
    }

    #[inline]
    pub fn generate_searcher(&self, id: usize) -> Searcher<P> {
        Searcher::new(
            id,
            self.board.clone(),
            self.evaluator.clone(),
            self.transposition_table.clone().into_inner(),
            self.num_nodes_searched.clone().into_inner(),
            self.selective_depth.clone().into_inner(),
            self.stop_command.clone().into_inner(),
            self.properties.clone(),
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

impl<T: SearchControl<Searcher<P>>, P: PositionEvaluation> ChessEngine for CustomEngine<T, P> {
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
        self.get_board_mut().set_fen(fen)?;
        self.reset_variables();
        Ok(())
    }

    fn set_transposition_table_size(&self, size: CacheTableSize) {
        self.transposition_table.set_size(size);
        if GLOBAL_TIMECAT_STATE.is_in_debug_mode() {
            self.transposition_table.print_info();
        }
    }

    #[inline]
    fn set_num_threads(&mut self, num_threads: NonZeroUsize) {
        self.num_threads = num_threads;
    }

    #[inline]
    fn set_move_overhead(&mut self, duration: Duration) {
        self.controller.set_move_overhead(duration);
    }

    #[inline]
    fn terminate(&self) -> bool {
        self.terminate.load(MEMORY_ORDERING)
    }

    #[inline]
    fn set_termination(&self, b: bool) {
        self.terminate.store(b, MEMORY_ORDERING);
    }

    fn clear_hash(&mut self) {
        self.get_transposition_table().clear();
        self.evaluator.clear();
    }

    fn print_info(&self) {
        print_engine_version();
        println_wasm!();
        self.transposition_table.print_info();
        self.evaluator.print_info();
    }

    #[inline]
    fn set_optional_io_reader(&mut self, optional_io_reader: Self::IoReader) {
        self.optional_io_reader = Some(optional_io_reader);
    }

    #[inline]
    fn evaluate_current_position(&mut self) -> Score {
        self.evaluator.evaluate(&self.board)
    }

    #[inline]
    fn evaluate_current_position_flipped(&mut self) -> Score {
        self.evaluator.evaluate_flipped(&self.board)
    }

    #[must_use = "If you don't need the search info, you can just search the position."]
    fn go(&mut self, command: &GoCommand, verbose: bool) -> SearchInfo {
        self.reset_variables();
        let mut join_handles = vec![];
        for id in 1..self.num_threads.get() {
            let mut threaded_searcher = self.generate_searcher(id);
            let controller = self.controller.clone();
            let join_handle = thread::spawn(move || {
                threaded_searcher.go(&GoCommand::Infinite, controller, false);
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

impl<T: SearchControl<Searcher<P>> + Default, P: PositionEvaluation + Default> CustomEngine<T, P> {
    #[inline]
    pub fn from_board(board: Board) -> Self {
        Self::new(
            board,
            TranspositionTable::default(),
            T::default(),
            P::default(),
        )
    }

    #[inline]
    pub fn from_fen(fen: &str) -> Result<Self> {
        Ok(Self::from_board(Board::from_fen(fen)?))
    }
}

impl<T: SearchControl<Searcher<P>>, P: PositionEvaluation> Clone for CustomEngine<T, P> {
    fn clone(&self) -> Self {
        Self {
            board: self.board.clone(),
            transposition_table: self.transposition_table.as_ref().clone().into(),
            evaluator: self.evaluator.clone(),
            controller: self.controller.clone(),
            num_nodes_searched: AtomicUsize::new(self.num_nodes_searched.load(MEMORY_ORDERING))
                .into(),
            selective_depth: AtomicUsize::new(self.selective_depth.load(MEMORY_ORDERING)).into(),
            optional_io_reader: self.optional_io_reader.clone(),
            stop_command: AtomicBool::new(self.stop_command.load(MEMORY_ORDERING)).into(),
            terminate: AtomicBool::new(self.terminate.load(MEMORY_ORDERING)).into(),
            properties: self.properties.clone(),
            ..*self
        }
    }
}

impl<T: SearchControl<Searcher<P>> + Default, P: PositionEvaluation + Default> Default
    for CustomEngine<T, P>
{
    fn default() -> Self {
        Self::new(
            Board::default(),
            TranspositionTable::default(),
            T::default(),
            P::default(),
        )
    }
}
