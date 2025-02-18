use shellfn::shell;
use std::error::Error;

#[shell(cmd = "python -c")]
fn pretty_json(json: &str, indent: u8, sort_keys: bool) -> Result<String, Box<dyn Error>> {
    r#"
import os, json

input = os.environ['JSON']
indent = int(os.environ['INDENT'])
sort_keys = os.environ['SORT_KEYS'] == 'true'
obj = json.loads(input)

print(json.dumps(obj, indent=indent, sort_keys=sort_keys))
    "#
}

fn main() -> Result<(), Box<dyn Error>> {
    let json = r#"{"foo": 42, "bar": { "baz": 10, "qux": [1, 2, 3]}}"#;
    let pretty_json = pretty_json(json, 2, false)?;
    println!("{}", pretty_json);
    Ok(())
}
