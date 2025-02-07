extern crate shellfn;

use shellfn::shell;
use std::error::Error as StdError;
use std::fmt::Display;

type BoxedError = Box<dyn StdError>;

#[test]
fn runs_simple_bash_script() {
    #[shell]
    fn subject() -> String { r#"
        echo -n "Hello, bash!"
    "# }

    assert_eq!("Hello, bash!", subject())
}

#[test]
fn runs_simple_python_script() {
    #[shell(cmd = "python -c")]
    fn subject() -> String { r#"
import sys
hello = lambda: "Hello, python!"
sys.stdout.write(hello())
    "# }

    assert_eq!("Hello, python!", subject())
}

#[test]
fn runs_script_with_program_on_custom_position() {
    #[shell(cmd = "python -c PROGRAM --world python")]
    fn subject() -> String { r#"
import sys
from argparse import ArgumentParser
parser = ArgumentParser()
parser.add_argument("-w", "--world", dest="world")
args = parser.parse_args()
sys.stdout.write("Hello, " + args.world + "!")
    "# }

    assert_eq!("Hello, python!", subject())
}

#[test]
fn sets_env_vars() {
    #[shell]
    fn subject(world: impl Display, foo: u32) -> String { r#"
        echo -n "Hello, $WORLD! The answer is $FOO"
    "# }

    assert_eq!("Hello, world! The answer is 42", subject("world", 42));
}

#[test]
fn replaces_env_vars_in_args() {
    #[shell(cmd = "bash -c \"echo -n Hello, $WORLD! The answer is $FOO\"")]
    fn subject(world: impl Display, foo: u32) -> String { "" }

    assert_eq!("Hello, world! The answer is 42", subject("world", 42));
}

#[test]
fn parses_return_value() {
    #[shell]
    fn join(x: u32, y: u32) -> u32 { r#"
        echo -n $X$Y
    "# }

    assert_eq!(4224, join(42, 24));
}

mod analyzes_return_type {
    use super::*;

    mod when_fn_does_not_return_anything {
        use super::*;

        mod and_fn_should_panic {
            use super::*;

            #[test]
            fn does_nothing_when_script_ends_with_success() {
                #[shell]
                fn script() { r#"
                    echo -n ""
                "# }

                script()
            }

            #[test]
            #[should_panic]
            fn panics_when_script_ends_with_failure() {
                #[shell]
                fn script() { r#"
                    exit 1
                "# }

                script()
            }

            #[test]
            #[should_panic]
            fn panics_when_script_is_invalid() {
                #[shell]
                fn script() { r#"
                    invalid script
                "# }

                script()
            }
        }

        mod and_fn_should_not_panic {
            use super::*;

            #[test]
            fn does_nothing_when_script_ends_with_success() {
                #[shell(no_panic)]
                fn script() { r#"
                    echo -n ""
                "# }

                script()
            }

            #[test]
            fn does_nothing_when_script_ends_with_failure() {
                #[shell(no_panic)]
                fn script() { r#"
                    exit 1
                "# }

                script()
            }

            #[test]
            fn does_nothing_when_script_is_invalid() {
                #[shell(no_panic)]
                fn script() { r#"
                    invalid script
                "# }

                script()
            }
        }
    }

    mod when_fn_returns_unit {
        use super::*;

        mod and_it_is_not_wrapped_with_result {
            use super::*;

            #[shell]
            fn script(interval: &str) -> () { r#"
                sleep $INTERVAL
            "# }

            #[shell(cmd = "dummy_invalid_command_123")]
            fn invalid_script() -> () { r#"
                invalid script
            "# }

            #[test]
            fn does_nothing_when_script_ends_with_success() {
                script("0")
            }

            #[test]
            #[should_panic]
            fn panics_when_script_ends_with_failure() {
                script("DEFINITELY_NOT_INT");
            }

            #[test]
            #[should_panic]
            fn panics_when_script_is_invalid() {
                invalid_script()
            }
        }

        mod and_it_is_wrapped_with_result {
            use super::*;

            #[shell]
            fn script(interval: &str) -> Result<(), BoxedError> { r#"
                sleep $INTERVAL
            "# }

            #[shell(cmd = "dummy_invalid_command_123")]
            fn invalid_script() -> Result<(), BoxedError> { r#"
                invalid script
            "# }

            #[test]
            fn returns_ok_when_script_ends_with_success() {
                assert!(script("0").is_ok())
            }

            #[test]
            fn returns_error_when_script_ends_with_failure() {
                assert!(script("DEFINITELY_NOT_INT").is_err())
            }

            #[test]
            fn returns_error_when_script_is_invalid() {
                assert!(invalid_script().is_err())
            }
        }
    }

    mod when_fn_returns_single_value {
        use super::*;

        mod and_it_is_not_wrapped_with_result {
            use super::*;

            #[shell]
            fn script(data: &str, exit_code: u32) -> u32 { r#"
                echo -n $DATA
                exit $EXIT_CODE
            "# }

            #[shell(cmd = "dummy_invalid_command_123")]
            fn invalid_script() -> u32 { r#"
                invalid script
            "# }

            #[test]
            fn returns_parsed_value_when_script_ends_with_success() {
                assert_eq!(42, script("42", 0))
            }

            #[test]
            #[should_panic]
            fn panics_when_parsing_fails() {
                let _ = script("DEFINITELY_NOT_INT", 0);
            }

            #[test]
            #[should_panic]
            fn panics_when_script_ends_with_failure() {
                let _ = script("42", 1);
            }

            #[test]
            #[should_panic]
            fn panics_when_script_is_invalid() {
                let _ = invalid_script();
            }
        }

        mod and_it_is_wrapped_with_result {
            use super::*;

            #[shell]
            fn script(data: &str, exit_code: u32) -> Result<u32, BoxedError> { r#"
                echo -n $DATA
                exit $EXIT_CODE
            "# }

            #[shell(cmd = "dummy_invalid_command_123")]
            fn invalid_script() -> Result<u32, BoxedError> { r#"
                invalid script
            "# }

            #[test]
            fn returns_parsed_value_when_script_ends_with_success() {
                assert_eq!(42, script("42", 0).unwrap())
            }

            #[test]
            fn returns_error_when_parsing_fails() {
                assert!(script("DEFINITELY_NOT_INT", 0).is_err())
            }

            #[test]
            fn returns_error_when_script_ends_with_failure() {
                assert!(script("42", 1).is_err())
            }

            #[test]
            fn returns_error_when_script_is_invalid() {
                assert!(invalid_script().is_err())
            }
        }
    }

    mod when_fn_returns_iterator {
        use super::*;

        mod and_it_is_not_wrapped_with_result {
            use super::*;

            mod and_the_item_is_not_wrapped_with_result {
                use super::*;

                mod and_fn_should_panic {
                    use super::*;

                    #[shell]
                    fn script(data: &str, exit_code: u32) -> impl Iterator<Item=u32> { r#"
                        for V in $DATA; do
                            echo $V;
                        done

                        exit $EXIT_CODE
                    "# }

                    #[shell(cmd = "dummy_invalid_command_123")]
                    fn invalid_script() -> impl Iterator<Item=u32> { r#"
                        invalid script iter
                    "# }

                    #[test]
                    fn returns_parsed_values_when_script_ends_with_success() {
                        let actual = script("42 100", 0).collect::<Vec<_>>();

                        assert_eq!(vec![42, 100], actual)
                    }

                    #[test]
                    #[should_panic]
                    fn panics_when_parsing_fails() {
                        let _ = script("100 DEFINITELY_NOT_INT", 0).collect::<Vec<_>>();
                    }

                    #[test]
                    #[should_panic]
                    fn panics_when_script_ends_with_failure() {
                        let _ = script("100", 1).collect::<Vec<_>>();
                    }

                    #[test]
                    #[should_panic]
                    fn panics_when_script_is_invalid() {
                        let _ = invalid_script().collect::<Vec<_>>();
                    }
                }

                mod and_fn_should_not_panic {
                    use super::*;

                    #[shell(no_panic)]
                    fn script(data: &str, exit_code: u32) -> impl Iterator<Item=u32> { r#"
                        for V in $DATA; do
                            echo $V;
                        done

                        exit $EXIT_CODE
                    "# }

                    #[shell(cmd = "dummy_invalid_command_123", no_panic)]
                    fn invalid_script() -> impl Iterator<Item=u32> { r#"
                        invalid script iter
                    "# }

                    #[test]
                    fn returns_only_successfuly_parsed_values_when_script_ends_with_success() {
                        assert_eq!(vec![42, 100], script("42 FOO 100 BAR", 0).collect::<Vec<_>>())
                    }

                    #[test]
                    fn returns_only_successfuly_parsed_values_when_script_ends_with_failure() {
                        assert_eq!(vec![42, 100], script("42 FOO 100", 1).collect::<Vec<_>>())
                    }

                    #[test]
                    fn returns_no_items_when_script_is_invalid() {
                        assert!(invalid_script().collect::<Vec<_>>().is_empty());
                    }
                }
            }

            mod and_the_item_is_wrapped_with_result {
                use super::*;

                mod and_fn_should_panic {
                    use super::*;

                    #[shell]
                    fn script(data: &str, exit_code: u32) -> impl Iterator<Item=Result<u32, BoxedError>> { r#"
                        for V in $DATA; do
                            echo $V;
                        done

                        exit $EXIT_CODE
                    "# }

                    #[shell(cmd = "dummy_invalid_command_123")]
                    fn invalid_script() -> impl Iterator<Item=Result<u32, BoxedError>> { r#"
                        invalid script iter
                    "# }

                    #[test]
                    fn returns_parsing_results_when_script_ends_with_success() {
                        let actual = script("42 FOO 100", 0).collect::<Vec<_>>();

                        assert_eq!(3, actual.len());
                        assert_eq!(42, *actual[0].as_ref().unwrap());
                        assert!(actual[1].is_err());
                        assert_eq!(100, *actual[2].as_ref().unwrap());
                    }

                    #[test]
                    #[should_panic]
                    fn panics_when_script_ends_with_failure() {
                        let _ = script("100", 1).collect::<Vec<_>>();
                    }

                    #[test]
                    #[should_panic]
                    fn panics_when_script_is_invalid() {
                        let _ = invalid_script().collect::<Vec<_>>();
                    }
                }

                mod and_fn_should_not_panic {
                    use super::*;

                    #[shell(no_panic)]
                    fn script(data: &str, exit_code: u32) -> impl Iterator<Item=Result<u32, BoxedError>> { r#"
                        for V in $DATA; do
                            echo $V;
                        done

                        exit $EXIT_CODE
                    "# }

                    #[shell(cmd = "dummy_invalid_command_123", no_panic)]
                    fn invalid_script() -> impl Iterator<Item=Result<u32, BoxedError>> { r#"
                        invalid script iter
                    "# }

                    #[test]
                    fn returns_parsing_results_when_script_ends_with_success() {
                        let actual = script("42 FOO 100 BAR", 0).collect::<Vec<_>>();

                        assert_eq!(4, actual.len());
                        assert_eq!(42, *actual[0].as_ref().unwrap());
                        assert!(actual[1].is_err());
                        assert_eq!(100, *actual[2].as_ref().unwrap());
                        assert!(actual[3].is_err());
                    }

                    #[test]
                    fn returns_parsing_results_when_script_ends_with_failure() {
                        let actual = script("42 FOO 100", 1).collect::<Vec<_>>();

                        assert_eq!(3, actual.len());
                        assert_eq!(42, *actual[0].as_ref().unwrap());
                        assert!(actual[1].is_err());
                        assert_eq!(100, *actual[2].as_ref().unwrap());
                    }

                    #[test]
                    fn returns_no_items_when_script_is_invalid() {
                        assert!(invalid_script().collect::<Vec<_>>().is_empty());
                    }
                }
            }
        }

        mod and_it_is_wrapped_with_result {
            use super::*;

            mod and_the_item_is_not_wrapped_with_result {
                use super::*;

                mod and_fn_should_panic {
                    use super::*;

                    #[shell]
                    fn script(data: &str, exit_code: u32) -> Result<impl Iterator<Item=u32>, BoxedError> { r#"
                        for V in $DATA; do
                            echo $V;
                        done

                        exit $EXIT_CODE
                    "# }

                    #[shell(cmd = "dummy_invalid_command_123")]
                    fn invalid_script() -> Result<impl Iterator<Item=u32>, BoxedError> { r#"
                        invalid script iter
                    "# }

                    #[test]
                    fn returns_parsed_values_when_script_ends_with_success() {
                        let actual = script("42 100", 0).unwrap().collect::<Vec<_>>();

                        assert_eq!(vec![42, 100], actual)
                    }

                    // warning: this is probably counter-intuitive but not sure if can be work-arounded
                    #[test]
                    fn returns_parsed_values_when_script_ends_with_failure() {
                        let actual = script("42 100", 1).unwrap().collect::<Vec<_>>();

                        assert_eq!(vec![42, 100], actual)
                    }

                    #[test]
                    #[should_panic]
                    fn panics_when_parsing_fails() {
                        let _ = script("100 DEFINITELY_NOT_INT", 0).map(|items| items.collect::<Vec<_>>());
                    }

                    #[test]
                    fn returns_error_when_script_is_invalid() {
                        assert!(invalid_script().is_err())
                    }
                }

                mod and_fn_should_not_panic {
                    use super::*;

                    #[shell(no_panic)]
                    fn script(data: &str, exit_code: u32) -> Result<impl Iterator<Item=u32>, BoxedError> { r#"
                        for V in $DATA; do
                            echo $V;
                        done

                        exit $EXIT_CODE
                    "# }

                    #[shell(cmd = "dummy_invalid_command_123", no_panic)]
                    fn invalid_script() -> Result<impl Iterator<Item=u32>, BoxedError> { r#"
                        invalid script iter
                    "# }

                    #[test]
                    fn returns_only_successfuly_parsed_values_when_script_ends_with_success() {
                        let actual = script("42 100 BAR", 0).unwrap().collect::<Vec<_>>();

                        assert_eq!(vec![42, 100], actual)
                    }

                    #[test]
                    fn returns_only_successfuly_parsed_values_when_script_ends_with_failure() {
                        let actual = script("42 FOO 100", 1).unwrap().collect::<Vec<_>>();

                        assert_eq!(vec![42, 100], actual)
                    }

                    #[test]
                    fn returns_error_when_script_is_invalid() {
                        assert!(invalid_script().is_err())
                    }
                }
            }

            mod and_the_item_is_wrapped_with_result {
                use super::*;

                #[shell]
                fn script(data: &str, exit_code: u32) -> Result<impl Iterator<Item=Result<u32, BoxedError>>, BoxedError> { r#"
                    for V in $DATA; do
                        echo $V;
                    done

                    exit $EXIT_CODE
                "# }

                #[shell(cmd = "dummy_invalid_command_123")]
                fn invalid_script() -> Result<impl Iterator<Item=Result<u32, BoxedError>>, BoxedError> { r#"
                    invalid script iter
                "# }

                #[test]
                fn returns_parsing_results_when_script_ends_with_success() {
                    let actual = script("BAR 42 FOO 100", 0).unwrap().collect::<Vec<_>>();

                    assert_eq!(4, actual.len());
                    assert!(actual[0].is_err());
                    assert_eq!(42, *actual[1].as_ref().unwrap());
                    assert!(actual[2].is_err());
                    assert_eq!(100, *actual[3].as_ref().unwrap());
                }

                #[test]
                fn returns_parsing_results_when_script_ends_with_failure() {
                    let actual = script("BAR 42 FOO 100", 1).unwrap().collect::<Vec<_>>();

                    assert_eq!(4, actual.len());
                    assert!(actual[0].is_err());
                    assert_eq!(42, *actual[1].as_ref().unwrap());
                    assert!(actual[2].is_err());
                    assert_eq!(100, *actual[3].as_ref().unwrap());
                }

                #[test]
                fn returns_error_when_script_is_invalid() {
                    assert!(invalid_script().is_err())
                }
            }
        }
    }

    mod when_fn_returns_vec {
        use super::*;

        mod and_it_is_not_wrapped_with_result {
            use super::*;

            mod and_the_item_is_not_wrapped_with_result {
                use super::*;

                mod and_fn_should_panic {
                    use super::*;

                    #[shell]
                    fn script(data: &str, exit_code: u32) -> Vec<u32> { r#"
                        for V in $DATA; do
                            echo $V;
                        done

                        exit $EXIT_CODE
                    "# }

                    #[shell(cmd = "dummy_invalid_command_123")]
                    fn invalid_script() -> Vec<u32> { r#"
                        invalid script iter
                    "# }

                    #[test]
                    fn returns_parsed_values_when_script_ends_with_success() {
                        let actual = script("42 100", 0);

                        assert_eq!(vec![42, 100], actual)
                    }

                    #[test]
                    #[should_panic]
                    fn panics_when_parsing_fails() {
                        script("100 DEFINITELY_NOT_INT", 0);
                    }

                    #[test]
                    #[should_panic]
                    fn panics_when_script_ends_with_failure() {
                        script("100", 1);
                    }

                    #[test]
                    #[should_panic]
                    fn panics_when_script_is_invalid() {
                        invalid_script();
                    }
                }

                mod and_fn_should_not_panic {
                    use super::*;

                    #[shell(no_panic)]
                    fn script(data: &str, exit_code: u32) -> Vec<u32> { r#"
                        for V in $DATA; do
                            echo $V;
                        done

                        exit $EXIT_CODE
                    "# }

                    #[shell(cmd = "dummy_invalid_command_123", no_panic)]
                    fn invalid_script() -> Vec<u32> { r#"
                        invalid script iter
                    "# }

                    #[test]
                    fn returns_only_successfuly_parsed_values_when_script_ends_with_success() {
                        assert_eq!(vec![42, 100], script("42 FOO 100 BAR", 0))
                    }

                    #[test]
                    fn returns_only_successfuly_parsed_values_when_script_ends_with_failure() {
                        assert_eq!(vec![42, 100], script("42 FOO 100", 1))
                    }

                    #[test]
                    fn returns_no_items_when_script_is_invalid() {
                        assert!(invalid_script().is_empty());
                    }
                }
            }

            mod and_the_item_is_wrapped_with_result {
                use super::*;

                mod and_fn_should_panic {
                    use super::*;

                    #[shell]
                    fn script(data: &str, exit_code: u32) -> Vec<Result<u32, BoxedError>> { r#"
                        for V in $DATA; do
                            echo $V;
                        done

                        exit $EXIT_CODE
                    "# }

                    #[shell(cmd = "dummy_invalid_command_123")]
                    fn invalid_script() -> Vec<Result<u32, BoxedError>> { r#"
                        invalid script iter
                    "# }

                    #[test]
                    fn returns_parsing_results_when_script_ends_with_success() {
                        let actual = script("42 FOO 100", 0);

                        assert_eq!(3, actual.len());
                        assert_eq!(42, *actual[0].as_ref().unwrap());
                        assert!(actual[1].is_err());
                        assert_eq!(100, *actual[2].as_ref().unwrap());
                    }

                    #[test]
                    #[should_panic]
                    fn panics_when_script_ends_with_failure() {
                        let _ = script("100", 1);
                    }

                    #[test]
                    #[should_panic]
                    fn panics_when_script_is_invalid() {
                        let _ = invalid_script();
                    }
                }

                mod and_fn_should_not_panic {
                    use super::*;

                    #[shell(no_panic)]
                    fn script(data: &str, exit_code: u32) -> Vec<Result<u32, BoxedError>> { r#"
                        for V in $DATA; do
                            echo $V;
                        done

                        exit $EXIT_CODE
                    "# }

                    #[shell(cmd = "dummy_invalid_command_123", no_panic)]
                    fn invalid_script() -> Vec<Result<u32, BoxedError>> { r#"
                        invalid script iter
                    "# }

                    #[test]
                    fn returns_parsing_results_when_script_ends_with_success() {
                        let actual = script("42 FOO 100 BAR", 0);

                        assert_eq!(4, actual.len());
                        assert_eq!(42, *actual[0].as_ref().unwrap());
                        assert!(actual[1].is_err());
                        assert_eq!(100, *actual[2].as_ref().unwrap());
                        assert!(actual[3].is_err());
                    }

                    #[test]
                    fn returns_parsing_results_when_script_ends_with_failure() {
                        let actual = script("42 FOO 100", 1);

                        assert_eq!(3, actual.len());
                        assert_eq!(42, *actual[0].as_ref().unwrap());
                        assert!(actual[1].is_err());
                        assert_eq!(100, *actual[2].as_ref().unwrap());
                    }

                    #[test]
                    fn returns_no_items_when_script_is_invalid() {
                        assert!(invalid_script().is_empty());
                    }
                }
            }
        }

        mod and_it_is_wrapped_with_result {
            use super::*;

            mod and_the_item_is_not_wrapped_with_result {
                use super::*;

                mod and_fn_should_panic {
                    use super::*;

                    #[shell]
                    fn script(data: &str, exit_code: u32) -> Result<Vec<u32>, BoxedError> { r#"
                        for V in $DATA; do
                            echo $V;
                        done

                        exit $EXIT_CODE
                    "# }

                    #[shell(cmd = "dummy_invalid_command_123")]
                    fn invalid_script() -> Result<Vec<u32>, BoxedError> { r#"
                        invalid script iter
                    "# }

                    #[test]
                    fn returns_parsed_values_when_script_ends_with_success() {
                        let actual = script("42 100", 0).unwrap();

                        assert_eq!(vec![42, 100], actual)
                    }

                    #[test]
                    fn returns_error_when_script_ends_with_failure() {
                        assert!(script("42 100", 1).is_err());
                    }

                    #[test]
                    #[should_panic]
                    fn panics_when_parsing_fails() {
                        let _ = script("100 DEFINITELY_NOT_INT", 0);
                    }

                    #[test]
                    fn returns_error_when_script_is_invalid() {
                        assert!(invalid_script().is_err())
                    }
                }

                mod and_fn_should_not_panic {
                    use super::*;

                    #[shell(no_panic)]
                    fn script(data: &str, exit_code: u32) -> Result<Vec<u32>, BoxedError> { r#"
                        for V in $DATA; do
                            echo $V;
                        done

                        exit $EXIT_CODE
                    "# }

                    #[shell(cmd = "dummy_invalid_command_123", no_panic)]
                    fn invalid_script() -> Result<Vec<u32>, BoxedError> { r#"
                        invalid script iter
                    "# }

                    #[test]
                    fn returns_only_successfuly_parsed_values_when_script_ends_with_success() {
                        let actual = script("42 100 BAR", 0).unwrap();

                        assert_eq!(vec![42, 100], actual)
                    }

                    #[test]
                    fn returns_error_when_script_ends_with_failure() {
                        assert!(script("42 FOO 100", 1).is_err())
                    }

                    #[test]
                    fn returns_error_when_script_is_invalid() {
                        assert!(invalid_script().is_err())
                    }
                }
            }

            mod and_the_item_is_wrapped_with_result {
                use super::*;

                #[shell]
                fn script(data: &str, exit_code: u32) -> Result<Vec<Result<u32, BoxedError>>, BoxedError> { r#"
                    for V in $DATA; do
                        echo $V;
                    done

                    exit $EXIT_CODE
                "# }

                #[shell(cmd = "dummy_invalid_command_123")]
                fn invalid_script() -> Result<Vec<Result<u32, BoxedError>>, BoxedError> { r#"
                    invalid script iter
                "# }

                #[test]
                fn returns_parsing_results_when_script_ends_with_success() {
                    let actual = script("BAR 42 FOO 100", 0).unwrap();

                    assert_eq!(4, actual.len());
                    assert!(actual[0].is_err());
                    assert_eq!(42, *actual[1].as_ref().unwrap());
                    assert!(actual[2].is_err());
                    assert_eq!(100, *actual[3].as_ref().unwrap());
                }

                #[test]
                fn returns_error_when_script_ends_with_failure() {
                    assert!(script("BAR 42 FOO 100", 1).is_err());
                }

                #[test]
                fn returns_error_when_script_is_invalid() {
                    assert!(invalid_script().is_err())
                }
            }
        }
    }
}
