use shellfn::shell;
use std::error::Error;

#[shell(cmd = "python -m $MODULE")]
fn run(module: &str) -> Result<String, Box<dyn Error>> {
    ""
}

fn main() -> Result<(), Box<dyn Error>> {
    run("calendar")
        .map(|output| println!("{}", output))
}
