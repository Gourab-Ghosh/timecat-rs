// https://github.com/glinscott/nnue-pytorch
// https://hxim.github.io/Stockfish-Evaluation-Guide/
// https://github.com/topics/nnue
// https://github.com/topics/chess-engine?l=rust
// https://github.com/dsekercioglu/blackmarlin.git
// https://github.com/zxqfl/sashimi
// https://backscattering.de/chess/uci/

mod tests;

use tests::test;
use timecat::*;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let clock = Instant::now();
    let args = env::args().collect_vec();
    if args.contains(&"--no-color".to_string()) {
        set_colored_output(false, false);
    }
    if args.contains(&"--uci".to_string()) {
        enable_uci_and_disable_color()
    }
    if args.contains(&"--test".to_string()) {
        test();
    } else if args.contains(&"-c".to_string()) || args.contains(&"--command".to_string()) {
        let index = args.iter().position(|s| s == "-c").unwrap_or(0)
            + args.iter().position(|s| s == "--command").unwrap_or(0)
            + 1;
        let command = args[index..].join(" ");
        let mut engine = Engine::default();
        println!();
        if let Err(err) = Parser::parse_command(&mut engine, &command) {
            let err_msg = err.stringify(Some(command.as_str()));
            println!("\n{}", colorize(err_msg, ERROR_MESSAGE_STYLE));
        }
    } else {
        let info_text = format!("{} {}", ENGINE_NAME, ENGINE_VERSION);
        println!("{}\n", colorize(info_text, SUCCESS_MESSAGE_STYLE));
        Parser::main_loop();
    }
    let elapsed_time = clock.elapsed().as_secs_f64();
    let precision = 3;
    println_info("\nRun Time", format!("{:.1$} s", elapsed_time, precision));
}
