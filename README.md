# shellfn
Rust proc macro for easily and safely use shell scripts in rust

# Example usage

## Basic usage
```
use shellfn::shellfn;

#[shell]
fn list_modified(dir: &str) -> Result<impl Iterator<Item=String>> { r#"
    cd $DIR
    git status | grep '^\s*modified:' | awk '{print $2}'
#" }
```

## Different interpreter
```
use shellfn::shellfn;
use std::error::Error;

#[shell(cmd = "python -c")]
fn pretty_json(json: &str, indent: u8, sort_keys: bool) -> Result<String, Box<Error>> { r#"
    import os, json

    json = os.environ['JSON']
    indent = os.environ['INDENT']
    sort_keys = os.environ['SORT_KEYS'] == 'true'
    obj = json.loads(json)

    print(json.dumps(obj, indent=indent, sort_keys=sort_keys))
 #" }
```
|                  return type                  |  flags   | on parse fail | on error exit code | on spawn fail | notes |
|-----------------------------------------------|----------|---------------|--------------------|---------------|-------|
|                                               |          | -             | panic              | panic         |       |
|                                               | no_panic | -             | nothing            | nothing       |       |
| T                                             |          | panic         | panic              | panic         | 2     |
| T                                             | no_panic | panic         | panic              | panic         | 1,2   |
| Result<T, E>                                  |          | error         | error              | error         | 2     |
| Result<T, E>                                  | no_panic | error         | error              | error         | 1,2   |
| impl Iterator<Item=T>                         |          | panic         | panic              | panic         |       |
| impl Iterator<Item=T>                         | no_panic | ignore errors | ignore errors      | empty iter    | 3     |
| impl Iterator<Item=Result<T, E>>              |          | item error    | panic              | panic         | 3     |
| impl Iterator<Item=Result<T, E>>              | no_panic | item error    | ignored            | empty iter    |       |
| Result<impl Iterator<Item=T>, E>              |          | panic         | ignored            | error         |       |
| Result<impl Iterator<Item=T>, E>              | no_panic | ignore errors | ignored            | error         |       |
| Result<impl Iterator<Item=Result<T, E1>>, E2> |          | item error    | ignored            | error         |       |
| Result<impl Iterator<Item=Result<T, E1>>, E2> | no_panic | item error    | ignored            | error         | 1     |

TO DO:
| Vec<T>                                        |          |               |                    |               |       |
| Vec<T>                                        | no_panic |               |                    |               |       |
| Result<Vec<T>, E>                             |          |               |                    |               |       |
| Result<Vec<T>, E>                             | no_panic |               |                    |               |       |

  Notes:

  1) `no_panic` attribute makes no difference
  2) it reads whole stdout first before any failure
  3) it yields all items until error exit code occurs

  Glossary:

  |     action    |                                 meaning                                  |
  |---------------|--------------------------------------------------------------------------|
  | panic         | panics (.expect or panic!)                                               |
  | nothing       | consumes and ignores error (let _ = ...)                                 |
  | error         | returns error                                                            |
  | ignore errors | yields all successfuly parsed items, ignores parsing failures (flat_map) |
  | empty iter    | returns empty iterator                                                   |
  | item error    | when parsing fails, yields Err                                           |
  | ignored       | ignores exit code, behaves in the same way for exit code 0 and != 0      |