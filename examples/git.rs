use shellfn::shell;
use std::error::Error;

#[shell]
fn list_modified(dir: &str) -> Result<impl Iterator<Item = String>, Box<Error>> { r#"
    cd $DIR
    git status | grep '^\s*modified:' | awk '{print $2}'
"# }

fn main() -> Result<(), Box<Error>> {
    for modified in list_modified(".")? {
        println!("You have modified the file: {}", modified);
    }
    Ok(())
}
