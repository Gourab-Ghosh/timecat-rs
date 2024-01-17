// https://github.com/glinscott/nnue-pytorch
// https://hxim.github.io/Stockfish-Evaluation-Guide/
// https://github.com/topics/nnue
// https://github.com/topics/chess-engine?l=rust
// https://github.com/dsekercioglu/blackmarlin.git
// https://github.com/zxqfl/sashimi
// https://backscattering.de/chess/uci/

use std::io::IsTerminal;
use timecat::chess_::*;
use timecat::*;

fn chess_test() {
    MoveGen::perft_test(&SubBoard::default(), 7);
}

fn main() {
    let args = std::env::args().collect_vec();
    let args = args.iter().map(|s| s.as_str()).collect_vec();
    if !args.contains(&"--disable-backtrace") {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    if args.contains(&"--test") {
        let clock = std::time::Instant::now();
        chess_test();
        let time_passed = clock.elapsed().as_secs_f64();
        println!("Time elapsed: {time_passed:.3} s");
        return;
    }
    if !std::io::stdin().is_terminal() {
        set_console_mode(false, false);
    }
    if !std::io::stdout().is_terminal() {
        set_colored_output(false, false);
    }
    Parser::parse_args_and_run_main_loop(&args);
}
