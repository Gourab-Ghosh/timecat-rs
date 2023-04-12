use super::*;

fn prediction_accuracy_func(rms: f32) -> f32 {
    1.0 - 1.0 / (1.0 + ((10.0 - rms) / 3.0).exp())
}

fn calculate_prediction_accuracy(rms: f32) -> f32 {
    (prediction_accuracy_func(rms) * 100.0) / prediction_accuracy_func(0.0)
}

fn self_play(engine: &mut Engine, go_command: GoCommand, print: bool, move_limit: Option<u16>) {
    let mut time_taken_vec: Vec<f32> = Vec::new();
    let mut max_time_taken_fen = String::new();
    let mut prediction_score_vec = Vec::new();
    println!("\n{}\n", engine.board);
    while !engine.board.is_game_over()
        && engine.board.get_fullmove_number() < move_limit.unwrap_or(u16::MAX)
    {
        let clock = Instant::now();
        if print {
            println!();
        }
        let (Some(best_move), score) = engine.go(go_command, print) else {panic!("No moves found")};
        let time_elapsed = clock.elapsed();
        let best_move_san = engine.board.san(Some(best_move)).unwrap();
        let pv = engine.get_pv_string();
        engine.push(Some(best_move));
        if time_elapsed.as_secs_f32()
            > *time_taken_vec
                .iter()
                .max_by(|&x, &y| x.partial_cmp(y).unwrap())
                .unwrap_or(&0.0)
        {
            max_time_taken_fen = engine.board.get_fen();
        }
        time_taken_vec.push(time_elapsed.as_secs_f32());
        prediction_score_vec.push(score as f32 / PAWN_VALUE as f32);
        let nps =
            (engine.get_num_nodes_searched() as u128 * 10u128.pow(9)) / time_elapsed.as_nanos();
        println!("\n{}\n", engine.board);
        println_info("Best Move", best_move_san);
        println_info("Score", score_to_string(score));
        println_info("Num Nodes Searched", engine.get_num_nodes_searched());
        println_info("PV Line", pv);
        println_info("Time Taken", format!("{:.3} s", time_elapsed.as_secs_f32()));
        println_info("Nodes per second", format!("{} nodes/s", nps));
    }
    let mean = time_taken_vec.iter().sum::<f32>() / time_taken_vec.len() as f32;
    let std_err = (time_taken_vec
        .iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f32>()
        / time_taken_vec.len() as f32)
        .sqrt();
    let max_time_taken = time_taken_vec
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let min_time_taken = time_taken_vec
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_abs_score = prediction_score_vec
        .iter()
        .map(|x| (x * 100.0).abs() as Score)
        .max()
        .unwrap();
    let min_abs_score = prediction_score_vec
        .iter()
        .map(|x| (x * 100.0).abs() as Score)
        .min()
        .unwrap();
    // let prediction_accuracy = prediction_score_vec.iter().filter(|x| x.abs() < 1.5).count() as f32 / prediction_score_vec.len() as f32 * 100.0;
    let prediction_score_rms = (prediction_score_vec.iter().map(|&x| x.powi(2)).sum::<f32>()
        / prediction_score_vec.len() as f32)
        .sqrt();
    let prediction_accuracy = calculate_prediction_accuracy(prediction_score_rms);
    println!(
        "\n{}:\n\n{}",
        colorize("Game PGN", INFO_STYLE),
        engine.board.get_pgn(),
    );
    println!(
        "\n{}:\n\n{:?}",
        colorize("Time taken for all moves", INFO_STYLE),
        time_taken_vec
            .iter()
            .map(|x| (x * 1000.0).round() / 1000.0)
            .collect_vec(),
    );
    println!(
        "\n{}:\n\n{:?}\n",
        colorize("Pediction Scores", INFO_STYLE),
        prediction_score_vec,
    );
    if let GoCommand::Depth(depth) = go_command {
        println_info("Depth Searched", format!("{}", depth));
    } else if let GoCommand::Time(time) = go_command {
        println_info(
            "Time Searched Per Move",
            format!("{:.3}", time.as_secs_f32()),
        );
    }
    println_info(
        "Time taken per move",
        format!("{:.3} \u{00B1} {:.3} s", mean, std_err),
    );
    println_info("Coefficient of Variation", format!("{:.3}", std_err / mean));
    println_info(
        "Prediction Score RMS",
        format!("{:.3}", prediction_score_rms),
    );
    println_info(
        "Prediction Accuracy",
        format!("{:.1} %", prediction_accuracy),
    );
    println_info("Max time taken", format!("{:.3} s", max_time_taken));
    println_info("Min time taken", format!("{:.3} s", min_time_taken));
    println_info("Max time taken by fen", max_time_taken_fen);
    println_info("Max prediction magnitude", score_to_string(max_abs_score));
    println_info("Min prediction magnitude", score_to_string(min_abs_score));
}

pub fn parse_command(engine: &mut Engine, raw_input: &str) {
    Parser::parse_command(engine, raw_input)
        .unwrap_or_else(|err| panic!("{}", err.generate_error(Some(raw_input))))
}

pub fn test() {
    // open_tablebase("directory", true, true, None, Board::new());
    let could_have_probably_played_better_move = [
        "5rk1/6pp/p1p5/1p1pqn2/1P6/2NP3P/2PQ1PP1/R5K1 w - - 0 26",
        "4b2k/N7/p1P1rn2/7p/1r1p1p1P/1P3P2/3K4/R2B2R1 b - - 0 42",
        "8/8/4K3/p7/P7/6kp/6p1/6Q1 w - - 0 70",
        "r1bqk1nr/ppp2ppp/2nb4/1B1pp3/5P2/1P2P3/PBPP2PP/RN1QK1NR b KQkq - 0 5",
        "7r/Q1pk1ppp/1p2p3/8/8/2q1BK2/P5PP/7R b - - 0 1",
        "B7/8/1p1knn2/pP4pp/P1KP1p1P/5P2/3B4/8 w - - 0 1",
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

    // let mut engine = Engine::default();
    // // engine.set_fen("8/8/8/1R5K/3k4/8/8/5rq1 b - - 1 96");
    // // engine.set_fen("7K/8/8/8/3k4/8/8/R7 w - - 15 57");
    // // engine.set_fen("k7/8/8/8/8/8/3P4/4K3 w - - 0 1"); // test endgame
    // // engine.set_fen("2kr1br1/p1pn1p2/2N1q2p/1PpQP3/5p1P/P6R/5PP1/2R3K1 w - - 2 30"); // check for repetitions
    // // engine.board.push_sans("e4 e5"); // e4 opwning
    // // engine.board.push_sans("e4 e6 d4 d5"); // caro cann defense
    // // engine.board.push_sans("d4 d5 c4"); // queens gambit
    // // engine.board.push_sans("d4 d5 c4 dxc4"); // queens gambit accepted
    // // engine.board.push_sans("e4 c5"); // sicilian defense
    // // engine.board.push_sans("e4 e5 Nf3 Nc6 Bc4 Nf6 Ng5"); // fried liver attack
    // // engine.board.push_sans("e4 e5 Nf3 Nc6 Bc4 Nf6 Ng5 Bc5"); // traxer counter attack
    // // engine.board.push_sans("e4 e5 Nf3 Nc6 Bc4 Nf6 Ng5 Bc5 Nxf7"); // traxer counter attack with Nxf7
    // // engine.set_fen("8/6k1/3r4/7p/7P/4R1P1/5P1K/8 w - - 3 59"); // endgame improvement 1
    // // engine.set_fen("8/7R/8/8/8/7K/k7/8 w - - 0 1"); // endgame improvement 2
    // // engine.set_fen("8/8/8/8/7P/8/6K1/3kr3 w - - 0 82"); // endgame improvement 3
    // // engine.set_fen("8/8/8/8/1K3k2/8/8/2r5 b - - 9 79"); // endgame improvement 4
    // // engine.set_fen("8/1K6/8/6R1/8/3k4/8/8 b - - 0 62"); // endgame improvement 4
    // // self_play(&mut engine, 16, false, Some(100));
    // self_play(&mut engine, GoCommand::Time(Duration::from_secs(3)), true, None);
    // // self_play(&mut engine, GoCommand::Depth(11), true, None);

    // parse_command(&mut Engine::default(), "go perft 7");

    let mut engine = Engine::default();
    // engine.set_fen("6k1/5p2/6p1/1K6/8/8/3r4/7q b - - 1 88"); // test if engine can find mate in 3
    // engine.set_fen("7R/r7/3K4/8/5k2/8/8/8 b - - 80 111"); // test t_table -> nodes initially: 3203606
    // engine.set_fen("8/8/K5k1/2q5/8/1Q6/8/8 b - - 20 105"); // gives incomplete pv line
    // engine.set_fen("k7/8/8/8/8/8/3P4/4K3 w - - 0 1"); // test endgame
    // engine.set_fen("4k2r/Q7/3b4/Q7/8/2N5/5PPP/5RK1 b - - 0 1"); // test draw by repetition
    // engine.set_fen(time_consuming_fens[7]);
    // engine.set_fen(could_have_probably_played_better_move[2]);
    // engine.set_fen("6k1/2N5/6b1/6p1/2p5/R1P1Bn1P/8/7K w - - 1 54"); // incomplete pv line in 3 secs in my pc
    // engine.set_fen("2r3k1/5pb1/2r1pnp1/q3P1B1/3P4/7R/2p2PP1/2Q2RK1 w - - 0 47"); // weird results in 3 secs in my pc
    // engine.set_fen("8/3k2P1/1p2Q3/3P4/4p3/2P1P3/6K1/q7 b - - 1 56"); // weird mating results in 3 secs in my pc
    engine.set_fen(could_have_probably_played_better_move[5]);
    // engine.board.push_sans("Qc6+ Kf2 Ra8 Rd1+ Ke8 Rc1 Qe4 Rxc7 Rxa7");
    // engine.set_fen("8/8/6K1/3k2P1/3b4/3N4/8/2B5 w - - 15 170");
    // engine.set_fen("3r2k1/4Rp1p/6q1/1N2p3/8/1PPr1P1b/4Q1PP/5RK1 w - - 1 24");
    parse_command(&mut engine, "go time 10000");
    // parse_command(&mut engine, "go depth 15");

    // let mut board = Board::new();
    // println!("\n{board}");
    // for san in ["e4", "Nf6", "Be2", "Nxe4"] {
    //     let move_ = board.parse_san(san).unwrap();
    //     let move_str = board.san(move_);
    //     println!("\nPushing move {move_str}");
    //     board.push(move_);
    //     println!("\n{board}");
    // }
}