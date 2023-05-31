// https://github.com/glinscott/nnue-pytorch
// https://hxim.github.io/Stockfish-Evaluation-Guide/
// https://github.com/topics/nnue
// https://github.com/topics/chess-engine?l=rust
// https://github.com/dsekercioglu/blackmarlin.git
// https://github.com/zxqfl/sashimi
// https://backscattering.de/chess/uci/

mod tests;

use timecat::*;
use tests::test;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    if ["windows"].contains(&env::consts::OS) {
        set_colored_output(false);
    }
    let clock = Instant::now();
    if env::args().contains(&String::from("--test")) {
        test();
    } else {
        let info_text = format!("Timecat {}", VERSION);
        println!("{}", colorize(info_text, SUCCESS_MESSAGE_STYLE));
        Parser::main_loop();
    }
    let elapsed_time = clock.elapsed().as_secs_f64();
    let precision = 3;
    println_info("\nRun Time", format!("{:.1$} s", elapsed_time, precision));
}
