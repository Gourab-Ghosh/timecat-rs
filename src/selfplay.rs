use std::f32::consts::E;

use crate::engine::SearchInfo;

use super::*;

fn prediction_accuracy_func(rms: f64) -> f64 {
    1.0 - 1.0 / (1.0 + ((10.0 - rms) / 3.0).exp())
}

fn calculate_prediction_accuracy(rms: f64) -> f64 {
    (prediction_accuracy_func(rms) * 100.0) / prediction_accuracy_func(0.0)
}

pub fn self_play(
    engine: &mut Engine,
    go_command: GoCommand,
    print: bool,
    move_limit: impl Into<Option<NumMoves>> + Copy,
) -> Result<(), EngineError> {
    let move_limit = move_limit.into().unwrap_or(NumMoves::MAX);
    if move_limit == 0 {
        return Ok(());
    }
    let stating_fen = engine.board.get_fen();
    let mut time_taken_vec: Vec<f64> = Vec::new();
    let mut max_time_taken_fen = String::new();
    let mut prediction_score_vec = Vec::new();
    println!("{}", engine.board);
    if engine.board.is_game_over() {
        return Err(EngineError::GameAlreadyOver);
    }
    let initial_num_moves = engine.board.get_num_moves();
    while !engine.board.is_game_over()
        && engine.board.get_num_moves() < initial_num_moves + move_limit
    {
        let clock = Instant::now();
        if print {
            println!();
        }
        let (Some(best_move), score) = engine.go(go_command, print) else {return Err(EngineError::BestMoveNotFound { fen: engine.board.get_fen() })};
        let time_elapsed = clock.elapsed();
        let best_move_san = best_move.stringify_move(&engine.board).unwrap();
        let pv = SearchInfo::get_pv_string(&engine.board, &engine.get_pv());
        engine.board.push(best_move);
        if time_elapsed.as_secs_f64()
            > *time_taken_vec
                .iter()
                .max_by(|&x, &y| x.partial_cmp(y).unwrap())
                .unwrap_or(&0.0)
        {
            max_time_taken_fen = engine.board.get_fen();
        }
        time_taken_vec.push(time_elapsed.as_secs_f64());
        prediction_score_vec.push(score);
        let nps =
            (engine.get_num_nodes_searched() as u128 * 10u128.pow(9)) / time_elapsed.as_nanos();
        println!("\n{}\n", engine.board);
        println_info("Best Move", best_move_san);
        println_info("Score", score.stringify_score());
        println_info("Num Nodes Searched", engine.get_num_nodes_searched());
        println_info("PV Line", pv);
        println_info("Time Taken", format!("{:.3} s", time_elapsed.as_secs_f64()));
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
    let max_abs_score = *prediction_score_vec.iter().max().unwrap();
    let min_abs_score = *prediction_score_vec.iter().min().unwrap();
    let prediction_score_rms = (prediction_score_vec
        .iter()
        .map(|&x| (x as f64).powi(2))
        .sum::<f64>()
        / prediction_score_vec.len() as f64)
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
        "\n{}:\n\n{}\n",
        colorize("Prediction Scores", INFO_STYLE),
        format!(
            "{:?}",
            prediction_score_vec
                .iter()
                .map(|&score| score.stringify_score())
                .collect_vec()
        )
        .replace('\"', ""),
    );
    if let GoCommand::Depth(depth) = go_command {
        println_info("Depth Searched", format!("{}", depth));
    } else if let GoCommand::MoveTime(time) = go_command {
        println_info(
            "Time Searched Per Move",
            format!("{:.3}", time.as_secs_f64()),
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
    println_info(
        "Max prediction magnitude",
        max_abs_score.stringify_score_normal(),
    );
    println_info(
        "Min prediction magnitude",
        min_abs_score.stringify_score_normal(),
    );
    engine.set_fen(&stating_fen)?;
    Ok(())
}
