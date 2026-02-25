use dep_why::{cli, run};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = cli::parse_args();
    match run(args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            // Print error message
            eprintln!("{}", e);
            // Use appropriate exit code per spec
            e.exit_code()
        }
    }
}
