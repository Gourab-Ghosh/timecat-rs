// https://github.com/glinscott/nnue-pytorch
// https://hxim.github.io/Stockfish-Evaluation-Guide/
// https://github.com/topics/nnue
// https://github.com/topics/chess-engine?l=rust
// https://github.com/dsekercioglu/blackmarlin.git
// https://github.com/zxqfl/sashimi
// https://backscattering.de/chess/uci/

fn main() {
    use std::io::IsTerminal;
    use timecat::*;

    let args = std::env::args().collect_vec();
    let args = args.iter().map(|s| s.as_str()).collect_vec();
    if !args.contains(&"--disable-backtrace") {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    if !std::io::stdin().is_terminal() {
        GLOBAL_TIMECAT_STATE.set_to_uci_mode();
    }
    #[cfg(feature = "colored")]
    if !std::io::stdout().is_terminal() {
        GLOBAL_TIMECAT_STATE.set_colored_output(false, false);
    }
    TimecatBuilder::<SearchController>::default()
        .parse_args(&args)
        .build()
        .run();
}
