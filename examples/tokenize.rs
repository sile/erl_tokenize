use erl_tokenize::{PositionRange, Tokenizer};
use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    src_file: String,
    #[structopt(long)]
    silent: bool,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let mut src = String::new();
    let mut file = File::open(opt.src_file).expect("Cannot open file");
    file.read_to_string(&mut src).expect("Cannot read file");

    let start_time = Instant::now();
    let mut count = 0;
    let tokenizer = Tokenizer::new(&src);
    for result in tokenizer {
        let token = result?;
        if !opt.silent {
            println!("[{:?}] {:?}", token.start_position(), token.text());
        }
        count += 1;
    }
    println!("TOKEN COUNT: {}", count);
    println!(
        "ELAPSED: {:?} seconds",
        to_seconds(Instant::now() - start_time)
    );
    Ok(())
}

fn to_seconds(duration: Duration) -> f64 {
    duration.as_secs() as f64 + f64::from(duration.subsec_nanos()) / 1_000_000_000.0
}
