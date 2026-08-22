use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> noargs::Result<ExitCode> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = "Tokenize all .erl/.hrl files under a directory";
    noargs::HELP_FLAG.take_help(&mut args);

    let root: PathBuf = noargs::arg("<ROOT_DIR>")
        .doc("Directory to scan recursively")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(ExitCode::SUCCESS);
    }

    let mut files = Vec::new();
    collect(&root, &mut files);
    files.retain(|p| !is_skipped(p));

    let mut err_files = 0usize;
    let mut err_total = 0usize;
    let mut tok_total = 0usize;
    let start = std::time::Instant::now();
    for path in &files {
        let src = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut count = 0;
        let mut errs = 0;
        let mut pos = erl_tokenize::Position::new();
        loop {
            match erl_tokenize::scan_token(&src, pos) {
                Ok(Some(t)) => {
                    count += 1;
                    pos = t.end();
                }
                Ok(None) => break,
                Err(e) => {
                    errs += 1;
                    eprintln!("{}: {}", path.display(), e);
                    pos = e.resume_position;
                }
            }
        }
        tok_total += count;
        err_total += errs;
        if errs > 0 {
            err_files += 1;
        }
    }
    println!(
        "FILES: {}\nOK FILES: {}\nFILES WITH ERRORS: {}\nTOTAL TOKENS: {}\nTOTAL ERRORS: {}\nELAPSED: {:?}",
        files.len(),
        files.len() - err_files,
        err_files,
        tok_total,
        err_total,
        start.elapsed(),
    );
    let _ = std::io::stdout().flush();
    Ok(ExitCode::from(u8::from(err_total > 0)))
}

fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect(&p, out);
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str())
                && (ext == "erl" || ext == "hrl")
            {
                out.push(p);
            }
        }
    }
}

/// Skip files that OTP itself does not treat as Erlang source: wxWidgets
/// binding templates (which embed `~s,` placeholder tokens for the code
/// generator, not valid Erlang) and files that contain literal NUL bytes
/// (`socket_sctp_SUITE.erl` in OTP master).
fn is_skipped(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.contains("/lib/wx/api_gen/wx_extra/") || s.ends_with("/lib/kernel/test/socket_sctp_SUITE.erl")
}
