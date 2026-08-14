use std::fs::File;
use std::io::Read;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use erl_tokenize::{Position, scan_token};

fn main() -> noargs::Result<ExitCode> {
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
        return Ok(ExitCode::SUCCESS);
    }

    let mut src = String::new();
    let mut file = File::open(&src_file).expect("cannot open source file");
    file.read_to_string(&mut src)
        .expect("cannot read source file");

    let start_time = Instant::now();
    let mut count = 0usize;
    let mut errors = 0usize;
    let mut pos = Position::new();
    loop {
        match scan_token(&src, pos) {
            Ok(Some(token)) => {
                if !silent {
                    println!("[{}] {:?}", token.start(), token.text(&src));
                }
                count += 1;
                pos = token.end();
            }
            Ok(None) => break,
            Err(error) => {
                eprintln!("{src_file}:{}: {error}", error.position);
                errors += 1;
                pos = error.resume_position;
            }
        }
    }
    println!("TOKEN COUNT: {count}");
    if errors > 0 {
        println!("ERROR COUNT: {errors}");
    }
    println!("ELAPSED: {:.6} seconds", to_seconds(start_time.elapsed()));
    Ok(if errors == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn to_seconds(duration: Duration) -> f64 {
    duration.as_secs() as f64 + f64::from(duration.subsec_nanos()) / 1_000_000_000.0
}
