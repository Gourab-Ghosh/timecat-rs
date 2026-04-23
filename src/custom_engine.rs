use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct EngineProperties {
    _use_mate_distance_pruning: bool,
    _clear_table_after_each_search: bool,
    _use_lmr: bool,
}

impl EngineProperties {
    #[inline]
    pub fn use_mate_distance_pruning(&self) -> bool {
        self._use_mate_distance_pruning && !DISABLE_ALL_PRUNINGS
    }

    #[inline]
    pub fn set_using_mate_distance_pruning(&mut self, value: bool) {
        self._use_mate_distance_pruning = value;
    }

    #[inline]
    pub fn clear_table_after_each_search(&self) -> bool {
        self._clear_table_after_each_search || DISABLE_ALL_PRUNINGS
    }

    #[inline]
    pub fn set_clearing_table_after_each_search(&mut self, value: bool) {
        self._clear_table_after_each_search = value;
    }

    #[inline]
    pub fn use_lmr(&self) -> bool {
        self._use_lmr && !DISABLE_ALL_PRUNINGS
    }

    #[inline]
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
pub struct CustomEngine<T, P: PositionEvaluation> {
    board: Board,
    transposition_table: TranspositionTable,
    evaluator: P,
    last_score: Option<Score>,
    controller: T,
    num_threads: NonZeroUsize,
    #[cfg_attr(feature = "serde", serde(skip))]
    optional_io_reader: Option<IoReader>,
    stop_command: Arc<AtomicBool>,
    terminate: Arc<AtomicBool>,
    properties: EngineProperties,
    #[cfg_attr(feature = "serde", serde(skip))]
    opening_book: Option<Arc<dyn PolyglotBook>>,
}

impl<T, P> CustomEngine<T, P>
where
    P: PositionEvaluation,
    for<'s> T: SearchControl<Searcher<'s, P>>,
{
    pub fn new(
        board: Board,
        transposition_table: TranspositionTable,
        controller: T,
        evaluator: P,
    ) -> Self {
        Self {
            board,
            transposition_table,
            evaluator,
            last_score: None,
            controller,
            num_threads: TIMECAT_DEFAULTS.num_threads,
            optional_io_reader: None,
            stop_command: AtomicBool::new(false).into(),
            terminate: AtomicBool::new(false).into(),
            properties: EngineProperties::default(),
            opening_book: TIMECAT_DEFAULTS
                .inbuilt_book_bytes
                .and_then(|bytes| PolyglotBookHashMap::try_from(bytes).ok())
                .map(|book| Arc::new(book) as Arc<dyn PolyglotBook>),
        }
    }

    #[inline]
    fn get_transposition_table(&self) -> &TranspositionTable {
        &self.transposition_table
    }

    // #[inline]
    // pub fn get_search_controller<'a, S : SearchControl<Searcher<'a, P>>>(&self) -> &S {
    //     &self.controller
    // }

    // #[inline]
    // pub fn get_search_controller_mut<'a, S : SearchControl<Searcher<'a, P>>>(
    //     &mut self,
    // ) -> &mut S {
    //     &mut self.controller
    // }

    #[inline]
    pub fn get_properties(&self) -> &EngineProperties {
        &self.properties
    }

    #[inline]
    pub fn get_properties_mut(&mut self) -> &mut EngineProperties {
        &mut self.properties
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

impl<T, P> ChessEngine for CustomEngine<T, P>
where
    P: PositionEvaluation,
    for<'s> T: SearchControl<Searcher<'s, P>>,
{
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

    fn set_transposition_table_size(&mut self, size: CacheTableSize) {
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
    fn get_opening_book(&self) -> Option<&dyn PolyglotBook> {
        self.opening_book.as_deref()
    }

    #[inline]
    fn set_opening_book<B: PolyglotBook + 'static>(&mut self, book: Option<Arc<B>>) {
        self.opening_book = book.map(|b| b as Arc<dyn PolyglotBook>);
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

    fn search(&mut self, config: &SearchConfig, verbose: bool) -> SearchInfo {
        if let Some(WeightedMove { move_, weight }) = self.get_opening_book_weighted_move() {
            return SearchInfoBuilder::new(self.board.get_position().clone(), vec![move_])
                .set_score(weight as Score)
                .build();
        }
        self.reset_variables();
        let mut search_info = std::thread::scope(|scope| {
            let mut join_handles = vec![];
            let num_nodes_searched = Arc::new(AtomicUsize::new(0));
            let selective_depth = Arc::new(AtomicUsize::new(0));

            let mut searchers = (0..self.num_threads.get()).map(|id| {
                Searcher::new(
                    id,
                    self.last_score,
                    self.board.clone(),
                    self.evaluator.clone(),
                    &self.transposition_table,
                    num_nodes_searched.clone(),
                    selective_depth.clone(),
                    &self.stop_command,
                    self.properties,
                )
            });

            // main thread searcher
            let mut main_thread_searcher = searchers.next().unwrap();

            // worker search threads
            searchers.for_each(|mut threaded_searcher| {
                let controller = self.controller.clone();
                join_handles.push(scope.spawn(move || {
                    threaded_searcher.search(&SearchConfig::new_infinite(), controller, false);
                }));
            });

            // stop/quit listener thread
            if let Some(io_reader) = self.optional_io_reader.as_ref() {
                let stop_command = self.stop_command.clone();
                let reader = io_reader.clone();
                let terminate = self.terminate.clone();

                join_handles.push(scope.spawn(move || {
                    Self::update_stop_command(stop_command, reader, terminate);
                }));
            }

            main_thread_searcher.search(config, self.controller.clone(), verbose);

            // signal everyone to stop; scoped threads will be joined automatically on scope exit
            self.set_stop_command(true);

            for join_handle in join_handles {
                join_handle.join().unwrap();
            }

            main_thread_searcher.get_search_info()
        });
        if search_info.get_pv().is_empty() && self.board.status() == BoardStatus::Ongoing {
            search_info.set_pv(vec![
                self.board
                    .get_single_legal_move(BitBoard::ALL, BitBoard::ALL)
                    .unwrap(),
            ]);
        }
        self.last_score = search_info.get_score();
        search_info
    }
}

impl<T, P> CustomEngine<T, P>
where
    P: PositionEvaluation + Default,
    for<'s> T: SearchControl<Searcher<'s, P>> + Default,
{
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

impl<T, P> Clone for CustomEngine<T, P>
where
    P: PositionEvaluation,
    for<'s> T: SearchControl<Searcher<'s, P>>,
{
    fn clone(&self) -> Self {
        Self {
            board: self.board.clone(),
            transposition_table: self.transposition_table.clone(),
            evaluator: self.evaluator.clone(),
            controller: self.controller.clone(),
            optional_io_reader: self.optional_io_reader.clone(),
            stop_command: AtomicBool::new(self.stop_command.load(MEMORY_ORDERING)).into(),
            terminate: AtomicBool::new(self.terminate.load(MEMORY_ORDERING)).into(),
            properties: self.properties,
            opening_book: self.opening_book.clone(),
            ..*self
        }
    }
}

impl<T, P> Default for CustomEngine<T, P>
where
    P: PositionEvaluation + Default,
    for<'s> T: SearchControl<Searcher<'s, P>> + Default,
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
