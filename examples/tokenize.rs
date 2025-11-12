use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

use erl_tokenize::{PositionRange, Tokenizer};

fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = "Tokenize Erlang source code";
    noargs::HELP_FLAG.take_help(&mut args);

    let silent: bool = noargs::flag("silent")
        .doc("Suppress token output")
        .take(&mut args)
        .is_present();
    let src_file: String = noargs::arg("<SRC_FILE>")
        .doc("Source file to tokenize")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let mut src = String::new();
    let mut file = File::open(src_file).expect("Cannot open file");
    file.read_to_string(&mut src).expect("Cannot read file");

    let start_time = Instant::now();
    let mut count = 0;
    let tokenizer = Tokenizer::new(&src);
    for result in tokenizer {
        let token = result?;
        if !silent {
            println!("[{:?}] {:?}", token.start_position(), token.text());
        }
        count += 1;
    }
    println!("TOKEN COUNT: {count}");
    println!(
        "ELAPSED: {:?} seconds",
        to_seconds(Instant::now() - start_time)
    );
    Ok(())
}

fn to_seconds(duration: Duration) -> f64 {
    duration.as_secs() as f64 + f64::from(duration.subsec_nanos()) / 1_000_000_000.0
}
