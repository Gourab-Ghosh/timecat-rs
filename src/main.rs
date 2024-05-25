// https://github.com/glinscott/nnue-pytorch
// https://hxim.github.io/Stockfish-Evaluation-Guide/
// https://github.com/topics/nnue
// https://github.com/topics/chess-engine?l=rust
// https://github.com/dsekercioglu/blackmarlin.git
// https://github.com/zxqfl/sashimi
// https://backscattering.de/chess/uci/

#[cfg(feature = "binary")]
fn main() {
    use std::io::IsTerminal;
    use timecat::*;

    let args = std::env::args().collect_vec();
    let args = args.iter().map(|s| s.as_str()).collect_vec();
    if !args.contains(&"--disable-backtrace") {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    if !std::io::stdin().is_terminal() {
        UCI_STATE.set_console_mode(false, false);
    }
    #[cfg(feature = "colored_output")]
    if !std::io::stdout().is_terminal() {
        UCI_STATE.set_colored_output(false, false);
    }
    Parser::parse_args_and_run_main_loop(&args);
}

#[cfg(not(feature = "binary"))]
fn main() {
    println!("Enable the Binary feature and Compile again!");
}
