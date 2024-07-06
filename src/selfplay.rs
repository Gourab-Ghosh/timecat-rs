use super::*;

#[cfg(feature = "debug")]
fn prediction_accuracy_func(rms: f64) -> f64 {
    1.0 - 1.0 / (1.0 + ((10.0 - rms) / 3.0).exp())
}

#[cfg(feature = "debug")]
fn calculate_prediction_accuracy(rms: f64) -> f64 {
    (prediction_accuracy_func(rms) * 100.0) / prediction_accuracy_func(0.0)
}

pub fn self_play(
    engine: &mut Engine,
    go_command: GoCommand,
    verbose: bool,
    move_limit: impl Into<Option<NumMoves>> + Copy,
) -> Result<()> {
    let move_limit = move_limit.into().unwrap_or(NumMoves::MAX);
    if move_limit == 0 {
        return Ok(());
    }
    let stating_fen = engine.get_board().get_fen();
    let mut time_taken_vec: Vec<f64> = Vec::new();
    let mut max_time_taken_fen = String::new();
    let mut prediction_score_vec = Vec::new();
    println_wasm!("{}", engine.get_board());
    if engine.get_board().is_game_over() {
        return Err(TimecatError::GameAlreadyOver);
    }
    let initial_num_moves = engine.get_board().get_num_moves();
    while !engine.get_board().is_game_over()
        && (engine.get_board().get_num_moves() as u64)
            < (initial_num_moves as u64) + (move_limit as u64)
    {
        let clock = Instant::now();
        if verbose {
            println_wasm!();
        }
        let response = engine.go(go_command, verbose);
        let Some(best_move) = response.get_best_move() else {
            return Err(TimecatError::BestMoveNotFound {
                fen: engine.get_board().get_fen(),
            });
        };
        let score = response.get_score();
        let time_elapsed = clock.elapsed();
        let best_move_san = best_move
            .stringify_move(engine.get_board().get_sub_board())
            .unwrap();
        let pv = get_pv_string(engine.get_board().get_sub_board(), response.get_pv());
        engine.get_board_mut().push_unchecked(best_move);
        if time_elapsed.as_secs_f64()
            > *time_taken_vec
                .iter()
                .max_by(|&x, &y| x.partial_cmp(y).unwrap())
                .unwrap_or(&0.0)
        {
            max_time_taken_fen = engine.get_board().get_fen();
        }
        time_taken_vec.push(time_elapsed.as_secs_f64());
        prediction_score_vec.push(score);
        let nps =
            (engine.get_num_nodes_searched() as u128 * 10u128.pow(9)) / time_elapsed.as_nanos();
        println_wasm!("\n{}\n", engine.get_board());
        println_info("Best Move", best_move_san);
        println_info("Score", score.stringify());
        println_info("Num Nodes Searched", engine.get_num_nodes_searched());
        println_info("PV Line", pv);
        println_info("Time Taken", time_elapsed.stringify());
        println_info("Nodes per second", format!("{} nodes/s", nps));
    }
    let mean = time_taken_vec.iter().sum::<f64>() / time_taken_vec.len() as f64;
    let std_err = (time_taken_vec
        .iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>()
        / time_taken_vec.len() as f64)
        .sqrt();
    let max_time_taken = time_taken_vec
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let min_time_taken = time_taken_vec
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    #[cfg(feature = "debug")]
    let max_abs_score = *prediction_score_vec.iter().max().unwrap();
    #[cfg(feature = "debug")]
    let min_abs_score = *prediction_score_vec.iter().min().unwrap();
    let prediction_score_rms = (prediction_score_vec
        .iter()
        .map(|&x| (x as f64).powi(2))
        .sum::<f64>()
        / prediction_score_vec.len() as f64)
        .sqrt();
    #[cfg(feature = "debug")]
    let prediction_accuracy = calculate_prediction_accuracy(prediction_score_rms);
    println_wasm!(
        "\n{}:\n\n{}",
        "Game PGN".colorize(INFO_MESSAGE_STYLE),
        engine.get_board().get_pgn(),
    );
    println_wasm!(
        "\n{}:\n\n[{}]",
        "Time taken for all moves".colorize(INFO_MESSAGE_STYLE),
        time_taken_vec
            .iter()
            .map(|x| (x * 1000.0).round() / 1000.0)
            .join(", "),
    );
    println_wasm!(
        "\n{}:\n\n[{}]\n",
        "Prediction Scores".colorize(INFO_MESSAGE_STYLE),
        prediction_score_vec
            .iter()
            .map(|&score| score.stringify())
            .join(", "),
    );
    if let GoCommand::Depth(depth) = go_command {
        println_info("Depth Searched", format!("{}", depth));
    } else if let GoCommand::MoveTime(time) = go_command {
        println_info("Time Searched Per Move", time.stringify());
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
    #[cfg(feature = "debug")]
    println_info(
        "Prediction Accuracy",
        format!("{:.1} %", prediction_accuracy),
    );
    println_info("Max time taken", format!("{:.3} s", max_time_taken));
    println_info("Min time taken", format!("{:.3} s", min_time_taken));
    println_info("Max time taken by fen", max_time_taken_fen);
    #[cfg(feature = "debug")]
    println_info("Max prediction magnitude", max_abs_score.stringify());
    #[cfg(feature = "debug")]
    println_info("Min prediction magnitude", min_abs_score.stringify());
    engine.set_fen(&stating_fen)?;
    Ok(())
}
