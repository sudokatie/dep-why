use dep_why::{cli, run};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = cli::parse_args();
    match run(args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            e.exit_code()
        }
    }
}
