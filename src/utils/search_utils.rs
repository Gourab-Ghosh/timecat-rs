use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct TimedGoCommand {
    pub wtime: Duration,
    pub btime: Duration,
    pub winc: Duration,
    pub binc: Duration,
    pub moves_to_go: Option<NumMoves>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum GoCommand {
    Ponder,
    Infinite,
    Limit {
        depth: Option<Depth>,
        nodes: Option<usize>,
        mate: Option<Ply>,
        movetime: Option<Duration>,
        time_clock: Option<TimedGoCommand>,
    },
}

impl GoCommand {
    pub const NONE: Self = Self::Limit {
        depth: None,
        nodes: None,
        mate: None,
        movetime: None,
        time_clock: None,
    };

    pub const fn from_depth(depth: Depth) -> Self {
        Self::Limit {
            depth: Some(depth),
            nodes: None,
            mate: None,
            movetime: None,
            time_clock: None,
        }
    }

    pub const fn from_nodes(nodes: usize) -> Self {
        Self::Limit {
            depth: None,
            nodes: Some(nodes),
            mate: None,
            movetime: None,
            time_clock: None,
        }
    }

    pub const fn from_mate(mate: Ply) -> Self {
        Self::Limit {
            depth: None,
            nodes: None,
            mate: Some(mate),
            movetime: None,
            time_clock: None,
        }
    }

    pub const fn from_movetime(movetime: Duration) -> Self {
        Self::Limit {
            depth: None,
            nodes: None,
            mate: None,
            movetime: Some(movetime),
            time_clock: None,
        }
    }

    pub const fn from_time_clock(
        wtime: Duration,
        btime: Duration,
        winc: Duration,
        binc: Duration,
        moves_to_go: Option<NumMoves>,
    ) -> Self {
        Self::Limit {
            depth: None,
            nodes: None,
            mate: None,
            movetime: None,
            time_clock: Some(TimedGoCommand {
                wtime,
                btime,
                winc,
                binc,
                moves_to_go,
            }),
        }
    }

    pub const fn from_millis(millis: u64) -> Self {
        Self::from_movetime(Duration::from_millis(millis))
    }

    pub fn has_infinite_config_info(&self) -> bool {
        *self == Self::Infinite
    }

    pub fn has_movetime_config_info(&self) -> bool {
        matches!(
            self,
            Self::Limit {
                movetime: Some(_),
                ..
            }
        )
    }

    pub fn has_depth_config_info(&self) -> bool {
        matches!(self, Self::Limit { depth: Some(_), .. })
    }

    pub fn has_time_clock_config_info(&self) -> bool {
        matches!(
            self,
            Self::Limit {
                time_clock: Some(_),
                ..
            }
        )
    }
}

#[derive(PartialEq, Eq, Default, Debug)]
struct LimitParser {
    depth: Option<Depth>,
    nodes: Option<usize>,
    mate: Option<Ply>,
    movetime: Option<Duration>,
    wtime: Option<Duration>,
    btime: Option<Duration>,
    winc: Option<Duration>,
    binc: Option<Duration>,
    moves_to_go: Option<NumMoves>,
}

impl TryInto<GoCommand> for LimitParser {
    type Error = TimecatError;

    fn try_into(self) -> std::result::Result<GoCommand, Self::Error> {
        if self.wtime.is_none() && self.btime.is_some() {
            return Err(TimecatError::WTimeNotMentioned);
        }
        if self.wtime.is_some() && self.btime.is_none() {
            return Err(TimecatError::BTimeNotMentioned);
        }
        let timed_go_command = match (self.wtime, self.btime) {
            (Some(wtime), Some(btime)) => Some(TimedGoCommand {
                wtime,
                btime,
                winc: self.winc.unwrap_or_default(),
                binc: self.binc.unwrap_or_default(),
                moves_to_go: self.moves_to_go,
            }),
            (None, Some(_)) => return Err(TimecatError::WTimeNotMentioned),
            (Some(_), None) => return Err(TimecatError::BTimeNotMentioned),
            (None, None) => None,
        };
        Ok(GoCommand::Limit {
            depth: self.depth,
            nodes: self.nodes,
            mate: self.mate,
            movetime: self.movetime,
            time_clock: timed_go_command,
        })
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SearchConfig {
    go_command: GoCommand,
    moves_to_search: Option<Vec<Move>>,
}

impl SearchConfig {
    #[inline]
    pub const fn from_go_command(go_command: GoCommand) -> Self {
        Self {
            go_command,
            moves_to_search: None,
        }
    }

    #[inline]
    pub const fn new_ponder() -> Self {
        Self::from_go_command(GoCommand::Ponder)
    }

    #[inline]
    pub const fn new_infinite() -> Self {
        Self::from_go_command(GoCommand::Infinite)
    }

    #[inline]
    pub const fn new_movetime(duration: Duration) -> Self {
        Self::from_go_command(GoCommand::from_movetime(duration))
    }

    #[inline]
    pub const fn new_depth(depth: Depth) -> Self {
        Self::from_go_command(GoCommand::from_depth(depth))
    }

    #[inline]
    pub const fn new_nodes(nodes: usize) -> Self {
        Self::from_go_command(GoCommand::from_nodes(nodes))
    }

    #[inline]
    pub const fn new_mate(mate: Ply) -> Self {
        Self::from_go_command(GoCommand::from_mate(mate))
    }

    #[inline]
    pub const fn get_go_command(&self) -> &GoCommand {
        &self.go_command
    }

    #[inline]
    pub fn set_go_command(&mut self, go_command: GoCommand) {
        self.go_command = go_command;
    }

    #[inline]
    pub fn get_moves_to_search(&self) -> Option<&[Move]> {
        self.moves_to_search.as_deref()
    }

    #[inline]
    pub fn set_moves_to_search(&mut self, moves: impl Into<Option<Vec<Move>>>) {
        self.moves_to_search = moves.into();
    }
}

impl Deref for SearchConfig {
    type Target = GoCommand;

    fn deref(&self) -> &Self::Target {
        &self.go_command
    }
}

impl From<GoCommand> for SearchConfig {
    #[inline]
    fn from(go_command: GoCommand) -> Self {
        Self::from_go_command(go_command)
    }
}

macro_rules! generate_command_in_error_message {
    ($commands:expr) => {
        TimecatError::InvalidGoCommand {
            s: $commands.join(" "),
        }
    };
}

impl TryFrom<&[&str]> for SearchConfig {
    type Error = TimecatError;

    fn try_from(commands: &[&str]) -> std::result::Result<Self, Self::Error> {
        let binding = commands
            .iter()
            .map(|command| command.to_lowercase())
            .collect_vec();
        let mut iter = binding.iter();
        let mut commands = vec![];
        let mut moves = vec![];
        let mut moves_cache = HashSet::new();
        for s in iter.by_ref() {
            if ["searchmove", "searchmoves"].contains(&s.as_str()) {
                break;
            }
            commands.push(s.as_str());
        }
        for s in iter {
            let move_ = Move::from_str(s)?;
            if moves_cache.insert(move_) {
                moves.push(move_);
            }
        }

        let second_command = commands
            .get(1)
            .ok_or(generate_command_in_error_message!(commands))?;
        if ["infinite", "ponder"].contains(second_command) && commands.get(2).is_some() {
            return Err(generate_command_in_error_message!(commands));
        }
        let go_command = match *second_command {
            "infinite" => GoCommand::Infinite,
            "ponder" => GoCommand::Ponder,
            _ => {
                let command_tuples = commands[1..]
                    .chunks(2)
                    .map(|chunk| {
                        if chunk.len() == 2 {
                            Ok((chunk[0], chunk[1]))
                        } else {
                            Err(generate_command_in_error_message!(commands))
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;
                if command_tuples.len() == 0 {
                    return Err(generate_command_in_error_message!(commands));
                }
                let mut limit_parser = LimitParser::default();
                for (command, value) in command_tuples {
                    match command {
                        "depth" => limit_parser.depth = Some(value.parse()?),
                        "nodes" => limit_parser.nodes = Some(value.parse()?),
                        "mate" => limit_parser.mate = Some(value.parse()?),
                        "movetime" => {
                            limit_parser.movetime = Some(Duration::from_millis(value.parse()?))
                        }
                        "wtime" => limit_parser.wtime = Some(Duration::from_millis(value.parse()?)),
                        "btime" => limit_parser.btime = Some(Duration::from_millis(value.parse()?)),
                        "winc" => limit_parser.winc = Some(Duration::from_millis(value.parse()?)),
                        "binc" => limit_parser.binc = Some(Duration::from_millis(value.parse()?)),
                        "movestogo" => limit_parser.moves_to_go = Some(value.parse()?),
                        _ => return Err(generate_command_in_error_message!(commands)),
                    }
                }
                limit_parser.try_into()?
            }
        };

        Ok(Self {
            go_command,
            moves_to_search: if moves.is_empty() { None } else { Some(moves) },
        })
    }
}

impl TryFrom<Vec<&str>> for SearchConfig {
    type Error = TimecatError;

    fn try_from(commands: Vec<&str>) -> std::result::Result<Self, Self::Error> {
        Self::try_from(commands.as_slice())
    }
}

impl FromStr for SearchConfig {
    type Err = TimecatError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let binding = remove_double_spaces_and_trim(s).to_lowercase();
        let commands = binding.split(' ').collect_vec();
        Self::try_from(commands)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct SearchInfoBuilder {
    position: BoardPosition,
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
    pub fn new(position: BoardPosition, pv: Vec<Move>) -> Self {
        Self {
            position,
            pv,
            ..Default::default()
        }
    }

    pub fn set_position(mut self, position: BoardPosition) -> Self {
        self.position = position;
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
            position: self.position,
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
    position: BoardPosition,
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
        position: BoardPosition,
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
            position,
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
        Some(self.position.score_flipped(self.get_score()?))
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
            Self::format_info(
                "score",
                self.get_score_flipped().map(|score| score.stringify()),
            ),
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
            Self::format_info("pv", Some(get_pv_string(&self.position, &self.pv))),
        ];
        println_wasm!("{}", outputs.into_iter().flatten().join(" "));
    }

    pub fn print_warning_message(&self, mut alpha: Score, mut beta: Score) {
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            alpha = self.position.score_flipped(alpha);
            beta = self.position.score_flipped(beta);
        }
        let warning_message = format!(
            "info string resetting alpha to -INFINITY and beta to INFINITY at depth {} having alpha {}, beta {} and score {} with time {}",
            if let Some(current_depth) = self.current_depth { current_depth.to_string() } else { STRINGIFY_NONE.to_string() },
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
        let (overwrites, collisions, zero_hit) = (
            Some(searcher.get_transposition_table().get_num_overwrites()),
            Some(searcher.get_transposition_table().get_num_collisions()),
            Some(searcher.get_transposition_table().get_zero_hit()),
        );
        #[cfg(not(feature = "extras"))]
        let (overwrites, collisions, zero_hit) = (None, None, None);
        let mut search_info = Self {
            position: searcher.get_initial_position().to_owned(),
            current_depth: Some(searcher.get_depth_completed().saturating_add(1)),
            seldepth: Some(searcher.get_selective_depth()),
            score: Some(searcher.get_score()),
            nodes: Some(searcher.get_num_nodes_searched()),
            hash_full: Some(searcher.get_transposition_table().get_hash_full()),
            overwrites,
            collisions,
            zero_hit,
            time_elapsed: Some(searcher.get_time_elapsed()),
            pv: searcher.get_pv().into_iter().copied().collect_vec(),
        };
        search_info.score = search_info
            .score
            .map(|score| search_info.position.score_flipped(score));
        search_info
    }
}
