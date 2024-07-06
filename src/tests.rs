use super::*;

static DUMMY_UCI_STATE_MANAGER: UCIStateManager = UCIStateManager::dummy();

pub fn parse_command(engine: &mut Engine, raw_input: &str) {
    Parser::parse_command(raw_input)
        .unwrap_or_else(|err| panic!("{}", err.stringify_with_optional_raw_input(Some(raw_input))))
        .into_iter()
        .for_each(|user_command| {
            user_command
                .run_command(engine, &DUMMY_UCI_STATE_MANAGER)
                .unwrap()
        });
}

#[allow(unused_variables)]
#[rustfmt::skip]
pub fn test(engine: &mut Engine) -> Result<()> {
    // open_tablebase("directory", true, true, None, Board::new());
    let could_have_probably_played_better_move = [
        "5rk1/6pp/p1p5/1p1pqn2/1P6/2NP3P/2PQ1PP1/R5K1 w - - 0 26",
        "4b2k/N7/p1P1rn2/7p/1r1p1p1P/1P3P2/3K4/R2B2R1 b - - 0 42",
        "8/8/4K3/p7/P7/6kp/6p1/6Q1 w - - 0 70",
        "r1bqk1nr/ppp2ppp/2nb4/1B1pp3/5P2/1P2P3/PBPP2PP/RN1QK1NR b KQkq - 0 5",
        "7r/Q1pk1ppp/1p2p3/8/8/2q1BK2/P5PP/7R b - - 0 1",
        "B7/8/1p1knn2/pP4pp/P1KP1p1P/5P2/3B4/8 w - - 0 1",
        "rnbqk1nr/pp3ppp/4p3/2ppP3/1b1P2Q1/2N5/PPP2PPP/R1B1KBNR b KQkq - 1 5",
    ];

    let time_consuming_fens = [
        "r2qrbk1/2p2ppp/b1p2n2/p2p4/4PB2/P1NB4/1PP2PPP/R2QR1K1 w - - 3 13",
        "2qr2k1/2p2pp1/2p4p/p3b3/8/P6P/1PPBQPP1/4R1K1 w - - 9 23",
        "r2qkb1r/p4pp1/2p4p/8/2n3n1/2NP4/PP2NPP1/R1BQK2R b KQkq - 1 14",
        "r2qk2r/p4pp1/2p4p/2b5/2n3n1/2NP4/PP2NPP1/R1BQK2R w KQkq - 2 15",
        "8/7R/8/8/8/8/2k3K1/8 w - - 4 3",
        "r3r3/3q1pk1/2pn2pp/pp1pR3/3P1P2/P6P/1P2QPP1/3NR1K1 b - - 10 33",
        "4b3/8/8/2K5/8/8/1k6/q7 w - - 0 115", // Taking really long to best move at depth 12
        "6k1/8/8/8/2q5/8/8/1K6 b - - 89 164", // Taking really long to best move at depth 12
        "5r2/5PK1/Pk6/5RP1/8/8/8/8 w - - 1 78", // Taking really long to best move at depth 12
        "8/8/8/8/1K6/5k2/8/5q2 b - - 1 75",   // Taking really long to best move at depth 12
        "8/8/q7/2K5/8/5k2/8/8 b - - 3 76",    // Taking really long to best move at depth 12
        "6R1/8/5K2/5N2/8/2k5/8/8 b - - 0 68", // Taking really long to best move at depth 14
        "1Q6/5pk1/8/4p3/8/6q1/3Q4/2K5 w - - 2 61", // Taking really long to best move at depth 12
        "r1bqr1k1/p1p2pp1/1b5p/3n4/2Q1N3/5N1P/PPP2PP1/R1B2RK1 b - - 2 16", // Taking really long to best move at depth 12
    ];

    // // engine.set_fen("8/8/8/1R5K/3k4/8/8/5rq1 b - - 1 96")?;
    // // engine.set_fen("7K/8/8/8/3k4/8/8/R7 w - - 15 57")?;
    // // engine.set_fen("k7/8/8/8/8/8/3P4/4K3 w - - 0 1")?; // test endgame
    // // engine.set_fen("2kr1br1/p1pn1p2/2N1q2p/1PpQP3/5p1P/P6R/5PP1/2R3K1 w - - 2 30")?; // check for repetitions
    // // engine.board.push_sans("e4 e5"); // e4 opening
    // // engine.board.push_sans("e4 e6 d4 d5"); // caro cann defense
    // // engine.board.push_sans("d4 d5 c4"); // queens gambit
    // // engine.board.push_sans("d4 d5 c4 dxc4"); // queens gambit accepted
    // // engine.board.push_sans("e4 c5"); // sicilian defense
    // // engine.board.push_sans("e4 e5 Nf3 Nc6 Bc4 Nf6 Ng5"); // fried liver attack
    // // engine.board.push_sans("e4 e5 Nf3 Nc6 Bc4 Nf6 Ng5 Bc5"); // traxler counter attack
    // // engine.board.push_sans("e4 e5 Nf3 Nc6 Bc4 Nf6 Ng5 Bc5 Nxf7"); // traxler counter attack with Nxf7
    // // engine.set_fen("8/6k1/3r4/7p/7P/4R1P1/5P1K/8 w - - 3 59")?; // endgame improvement 1
    // // engine.set_fen("8/7R/8/8/8/7K/k7/8 w - - 0 1")?; // endgame improvement 2
    // // engine.set_fen("8/2p5/2k5/8/2K5/8/8/7n b - - 16 8")?; // endgame improvement 2
    // engine.set_fen("k6B/8/8/8/8/8/8/K6N w - - 0 1")?; // knight bishop endgame
    // // engine.set_fen("k6B/8/8/4N3/8/8/8/K6N w - - 0 1")?; // knight knight bishop endgame
    // // engine.set_fen("k6N/8/8/8/8/8/8/K6N w - - 0 1")?; // 2 knights endgame
    // // engine.set_fen("k6N/8/8/8/4N3/8/8/K6N w - - 0 1")?; // 3 knights endgame
    // // engine.set_fen("8/8/8/8/8/8/6KP/3kr3 w - - 0 82")?; // endgame improvement 3
    // // engine.set_fen("4k3/R7/8/3KP3/8/6r1/8/8 b - - 0 1")?; // endgame improvement 3
    // // engine.set_fen("8/p7/2Q3pp/4Pk2/P7/2b5/Kp6/4r3 w - - 26 108")?; // perpetual check
    // // self_play(engine, 16, false, 100);
    // self_play(engine, GoCommand::MoveTime(Duration::from_secs(3)), true, None)?;
    // // self_play(engine, GoCommand::Depth(11), true, None)?;

    // self_play(&mut Engine::from_fen("8/8/8/8/2N5/B2K4/8/1k6 b - - 73 37")?, GoCommand::MoveTime(Duration::from_secs(1)), true, 2)?;
    // self_play(&mut Engine::from_fen("7k/8/8/5Ppp/1pB1P3/1P2B3/5KP1/8 w - - 0 56")?, GoCommand::MoveTime(Duration::from_secs(3)), true, None)?;
    // self_play(&mut Engine::from_fen("8/8/5Q2/7p/1pBBP1k1/1P4p1/4K1P1/8 w - - 3 62")?, GoCommand::MoveTime(Duration::from_secs(3)), true, None)?;

    // let mut halfkp_model = HALFKP_MODEL_READER.to_default_model();
    // halfkp_model.deactivate_non_king_piece(White, WhitePawn, E2);
    // halfkp_model.deactivate_non_king_piece(Black, WhitePawn, E2);
    // halfkp_model.activate_non_king_piece(White, WhitePawn, E4);
    // halfkp_model.activate_non_king_piece(Black, WhitePawn, E4);
    // println_wasm!("{:#?}", halfkp_model.evaluate(Black));

    // println_wasm!("{}", Board::from_fen("8/8/8/8/7R/7K/k7/8 b - - 1 1")?);

    // parse_command(Engine::default(), "go perft 7");

    // let mut engine = Engine::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")?;
    // parse_command(engine, "go perft 6");

    // GLOBAL_TIMECAT_STATE.set_num_threads(2, true);
    // engine.set_fen("6k1/5p2/6p1/1K6/8/8/3r4/7q b - - 1 88")?; // test if engine can find mate in 3
    // engine.set_fen("7R/r7/3K4/8/5k2/8/8/8 b - - 80 111")?; // test t_table -> nodes initially: 3203606
    // engine.set_fen("8/8/K5k1/2q5/8/1Q6/8/8 b - - 20 105")?; // gives incomplete pv line
    // engine.set_fen("k7/8/8/8/8/8/3P4/4K3 w - - 0 1")?; // test endgame
    // engine.set_fen("4k2r/Q7/3b4/Q7/8/2N5/5PPP/5RK1 b - - 0 1")?; // test draw by repetition
    // engine.set_fen(time_consuming_fens[7])?;
    // engine.set_fen(could_have_probably_played_better_move[2])?;
    // engine.set_fen("6k1/2N5/6b1/6p1/2p5/R1P1Bn1P/8/7K w - - 1 54")?; // incomplete pv line in 3 secs in my pc
    // engine.set_fen("2r3k1/5pb1/2r1pnp1/q3P1B1/3P4/7R/2p2PP1/2Q2RK1 w - - 0 47")?; // weird results in 3 secs in my pc
    // engine.set_fen("8/3k2P1/1p2Q3/3P4/4p3/2P1P3/6K1/q7 b - - 1 56")?; // weird mating results in 3 secs in my pc
    // engine.set_fen("8/R1pk3p/8/4B2p/p1r5/8/6PK/8 w - - 0 41")?; // weird mating results in 3 secs in my pc
    // engine.set_fen(could_have_probably_played_better_move[6])?;
    // engine.board.push_sans("Qc6+ Kf2 Ra8 Rd1+ Ke8 Rc1 Qe4 Rxc7 Rxa7")?;
    // engine.set_fen("8/8/6K1/3k2P1/3b4/3N4/8/2B5 w - - 15 170")?;
    // engine.set_fen("3r2k1/4Rp1p/6q1/1N2p3/8/1PPr1P1b/4Q1PP/5RK1 w - - 1 24")?;
    // engine.set_fen("8/5K1k/2n5/2N5/6P1/8/8/B7 w - - 11 170")?; // check for saving mate score
    // engine.set_fen("r2qr1k1/p1p2ppp/2P5/3n4/1b4b1/2N2P2/PPP1B1PP/R1BQK2R w KQ - 3 12")?; // weird results in 3 secs in my pc
    // engine.set_fen("6k1/p2b1ppp/3r4/6q1/1p2Pb2/1N1B3P/PP2QPP1/4R1K1 w - - 1 33")?; // Missed tactics in 3 sec move
    // parse_command(engine, "go movetime 3000");
    parse_command(engine, "go depth 15");

    // println_wasm!("{}", BitBoard::new(123456));

    // let mut all_optional_pieces = vec![None];
    // all_optional_pieces.extend_from_slice(&ALL_PIECE_TYPES.map(|piece| Some(piece)));
    // for source in ALL_SQUARES {
    //     for dest in ALL_SQUARES {
    //         for &promotion in &all_optional_pieces {
    //             let valid_or_null_move = ValidOrNullMove::new(source, dest, promotion);
    //             let compressed_then_decompressed_move: Option<ValidOrNullMove> = valid_or_null_move.compress().decompress();
    //             let compressed_then_decompressed_move = compressed_then_decompressed_move.unwrap();
    //             if valid_or_null_move != compressed_then_decompressed_move {
    //                 // println_wasm!("{valid_or_null_move}: {valid_or_null_move:?} ----- {compressed_then_decompressed_move:?}");
    //                 println_wasm!("{valid_or_null_move} ----- {} ----- {compressed_then_decompressed_move}", valid_or_null_move.compress());
    //             }
    //         }
    //     }
    // }

    // let mut sans =  "Nc3 Nf6 d4 d5 e3 Nc6 Nf3 Bg4 h3 Bh5 g4 Bg6 Bb5 a6 Bxc6+ bxc6 Ne5 Qd6 h4 Ne4 h5 Nxc3 bxc3 Be4 f3 f6 fxe4 fxe5 Qf3 dxe4 Qxe4 exd4 cxd4 O-O-O Ke2 Qd5 Kd3 Qxe4+ Kxe4 e6 c4 c5 Bb2 cxd4 Bxd4 Rg8 Raf1 g6 h6 Bb4 Rf7 Rd7 Rg7 Re8 Rf1 e5 Bb2 Rd2 Rff7 Bd6 c5 Rxb2 cxd6 cxd6 Rxh7 Kb8 Rhg7 Rd8 Rxg6 Rxa2 h7 d5+ Kxe5 Ra3 Rg8 Rxe3+ Kf4 Re4+ Kg5 Ree8 Rgg7 Re2 Kh4 Rh2+ Kg5 Re2 Kh4 Rh2+ Kg5 Re2".split(' ').collect_vec();
    // for san in &mut sans[0..76] {
    //     engine.board.push_san(san);
    // }
    // println_wasm!("{}", engine.board);
    // let go_command = GoCommand::Timed {
    //     wtime: Duration::from_millis(267809),
    //     btime: Duration::from_millis(532920),
    //     winc: Duration::from_millis(0),
    //     binc: Duration::from_millis(0),
    //     moves_to_go: None,
    // };
    // engine.go(go_command, true);

    // let path = "";
    // test_polyglot(path)?;

    // let mut board = Board::new();
    // println_wasm!("\n{board}");
    // for san in ["e4", "Nf6", "Be2", "Nxe4"] {
    //     let valid_or_null_move = board.parse_stringify_move(san)?;
    //     let move_str = board.stringify_move(valid_or_null_move);
    //     println_wasm!("\nPushing move {move_str}");
    //     board.push(valid_or_null_move);
    //     println_wasm!("\n{board}");
    // }

    // let mut board = Board::default();
    // board.set_fen("8/8/8/p1ppkPp1/P1p3P1/2P1K3/2P5/8 b - - 0 36").unwrap();
    // let moves = "d4+ cxd4+ cxd4+ Kd2 Kf6 c3 d3 Ke3 Ke5 Kd2 Kd5 Ke3 Ke5 Kd2 Kf6".split(' ');
    // for move_san in moves {
    //     let valid_or_null_move = board.parse_san(move_san).unwrap();
    //     let gives_repetition = board.gives_repetition(valid_or_null_move.unwrap());
    //     board.push(valid_or_null_move);
    //     println_wasm!("{move_san}: {gives_repetition} {} {}", board.get_num_repetitions(), board.get_hash().stringify());
    // }

    Ok(())
}
