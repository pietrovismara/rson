#[macro_use]
extern crate serde_json;
extern crate clap;

mod predicate;

use predicate::{parsers, operators};
use clap::{App, Arg, SubCommand, ArgMatches};
use std::io::{self, Write, Read};
use std::collections::HashMap;
use std::error::Error;
use std::process;
use std::fs::File;

fn main() {
    let matches = App::new("myapp")
    .subcommand(
        SubCommand::with_name("read")
        .about("Reads a json file")
        .arg(
            Arg::with_name("input")
            .help("the file to add")
            .required(true)
        )
    )
    .subcommand(
        SubCommand::with_name("pick")
        .about("Pick one or more properties or indexes")
        .arg(
            Arg::with_name("keys")
            .help("Property keys to pick")
            .multiple(true)
            .required(true)
        )
    )
    .subcommand(
        SubCommand::with_name("get")
        .about("Get value found at given path")
        .arg(
            Arg::with_name("path")
            .help("Path of the value to get")
            .required(true)
        )
    )
    .subcommand(
        SubCommand::with_name("filter")
        .about("Filter elements of given array based on predicate")
        .arg(
            Arg::with_name("predicate")
            .help("Fn to execute over iteration")
            .required(true)
        )
    )
    .subcommand(
        SubCommand::with_name("some")
        .about("Returns true if at least one element in the given array matches the given predicate")
        .arg(
            Arg::with_name("predicate")
            .help("Fn to execute over iteration")
            .required(true)
        )
    )
    .subcommand(
        SubCommand::with_name("every")
        .about("Returns true if at least one element in the given array matches the given predicate")
        .arg(
            Arg::with_name("predicate")
            .help("Fn to execute over iteration")
            .required(true)
        )
    )
    .subcommand(
        SubCommand::with_name("find")
        .about("Returns the first element that matches given predicate in given array ")
        .arg(
            Arg::with_name("predicate")
            .help("Fn to execute over iteration")
            .required(true)
        )
    )
    .subcommand(
        SubCommand::with_name("length")
        .about("Returns length of given array")
    )
    .subcommand(
        SubCommand::with_name("pretty")
        .about("Prints given json in a readable format.")
    )
    .get_matches();

    match matches.subcommand_name() {
        Some("read")  => {
            read(matches).unwrap_or_else(|e| {
                println!("{}", e);
                process::exit(1);
            });
        },
        Some("pick") => {
            let result = pick(matches).unwrap_or_else(|e| {
                println!("{}", e);
                process::exit(1);
            });
            write_result(result);
        },
        Some("get") => {
            let result = get(matches).unwrap_or_else(|e| {
                println!("{}", e);
                process::exit(1);
            });
            write_result(result);
        },
        Some("filter") => {
            let result = filter(matches).unwrap_or_else(|e| {
                println!("{}", e);
                process::exit(1);
            });
            write_result(result);
        },
        Some("some") => {
            let result = some(matches).unwrap_or_else(|e| {
                println!("{}", e);
                process::exit(1);
            });
            write_result(result);
        },
        Some("every") => {
            let result = every(matches).unwrap_or_else(|e| {
                println!("{}", e);
                process::exit(1);
            });
            write_result(result);
        },
        Some("find") => {
            let result = find(matches).unwrap_or_else(|e| {
                println!("{}", e);
                process::exit(1);
            });
            write_result(result);
        },
        Some("length") => {
            let result = length(matches).unwrap_or_else(|e| {
                println!("{}", e);
                process::exit(1);
            });
            write_result(result);
        },
        Some("pretty") => {
            let result = pretty().unwrap_or_else(|e| {
                println!("{}", e);
                process::exit(1);
            });
            write_result(result);
        },
        _ => {}, // Either no subcommand or one not tested for...
    }
}

fn read_stdin() -> Result<String, Box<Error>> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    Ok(buffer)
}

fn write_result(result: String) {
    io::stdout().write(result.as_bytes()).unwrap_or_default();
    io::stdout().write("\n".as_bytes()).unwrap_or_default();
}

fn pretty() -> Result<String, Box<Error>> {
    let buff: String = read_stdin()?;
    let v: serde_json::Value = serde_json::from_str(&buff)?;

    Ok(serde_json::to_string_pretty(&v)?)
}

fn pick(matches: ArgMatches) -> Result<String, Box<Error>> {
    let v: serde_json::Value = prepare_object_data()?;

    let submatches = matches.subcommand_matches("pick").unwrap();
    let keys: Vec<&str> = submatches.values_of("keys").unwrap().collect();
    let mut results = HashMap::new();

    for key in keys {
        results.insert(String::from(key), v.get(key));
    }

    Ok(serde_json::to_string(&results)?)
}

fn get(matches: ArgMatches) -> Result<String, Box<Error>> {
    let v: serde_json::Value = prepare_object_data()?;

    let submatches = matches.subcommand_matches("get").unwrap();
    let key: &str = submatches.value_of("path").unwrap();
    let path = parsers::parse_key(key);
    let val = v.pointer(&path);

    Ok(serde_json::to_string(&val)?)
}

fn filter(matches: ArgMatches) -> Result<String, Box<Error>> {
    let v = prepare_array_data()?;
    let v = v.as_array().unwrap();
    let (path, operator, value) = prepare_array_op(matches, "filter")?;

    let filtered: Vec<&serde_json::Value> = v.iter().filter(|elt| {
        let val = cast_path(elt, &path);
        if let Some(s) = val.as_str() {
            if let parsers::Value::Str(ref s2) = value {
                return compare(s, s2, &operator);
            }
        } else if let Some(n) = val.as_f64() {
            if let parsers::Value::Num(n2) = value {
                return compare(n, n2, &operator);
            }
        } else if let Some(b) = val.as_bool() {
            if let parsers::Value::Bool(b2) = value {
                return compare(b, b2, &operator);
            }
        }

        false
    }).collect();

    Ok(serde_json::to_string(&filtered)?)
}

fn some(matches: ArgMatches) -> Result<String, Box<Error>> {
    let v = prepare_array_data()?;
    let v = v.as_array().unwrap();
    let (path, operator, value) = prepare_array_op(matches, "some")?;

    let any: bool = v.iter().any(|elt| {
        let val = cast_path(elt, &path);
        if let Some(s) = val.as_str() {
            if let parsers::Value::Str(ref s2) = value {
                return compare(s, s2, &operator);
            }
        } else if let Some(n) = val.as_f64() {
            if let parsers::Value::Num(n2) = value {
                return compare(n, n2, &operator);
            }
        } else if let Some(b) = val.as_bool() {
            if let parsers::Value::Bool(b2) = value {
                return compare(b, b2, &operator);
            }
        }

        false
    });

    Ok(serde_json::to_string(&any)?)
}

fn every(matches: ArgMatches) -> Result<String, Box<Error>> {
    let v = prepare_array_data()?;
    let v = v.as_array().unwrap();
    let (path, operator, value) = prepare_array_op(matches, "every")?;

    let every: bool = v.iter().all(|elt| {
        let val = cast_path(elt, &path);
        if let Some(s) = val.as_str() {
            if let parsers::Value::Str(ref s2) = value {
                return compare(s, s2, &operator);
            }
        } else if let Some(n) = val.as_f64() {
            if let parsers::Value::Num(n2) = value {
                return compare(n, n2, &operator);
            }
        } else if let Some(b) = val.as_bool() {
            if let parsers::Value::Bool(b2) = value {
                return compare(b, b2, &operator);
            }
        }

        false
    });

    Ok(serde_json::to_string(&every)?)
}

fn find(matches: ArgMatches) -> Result<String, Box<Error>> {
    let v = prepare_array_data()?;
    let v = v.as_array().unwrap();
    let (path, operator, value) = prepare_array_op(matches, "find")?;

    let result: Option<&serde_json::Value> = v.iter().find(|elt| {
        let val = cast_path(elt, &path);
        if let Some(s) = val.as_str() {
            if let parsers::Value::Str(ref s2) = value {
                return compare(s, s2, &operator);
            }
        } else if let Some(n) = val.as_f64() {
            if let parsers::Value::Num(n2) = value {
                return compare(n, n2, &operator);
            }
        } else if let Some(b) = val.as_bool() {
            if let parsers::Value::Bool(b2) = value {
                return compare(b, b2, &operator);
            }
        }

        false
    });

    match result {
        Some(r) => Ok(serde_json::to_string(&r)?),
        None => Ok(serde_json::to_string(&json!(null))?)
    }
}

fn length(_matches: ArgMatches) -> Result<String, Box<Error>> {
    let v = prepare_array_data()?;
    let v = v.as_array().unwrap();    

    Ok(serde_json::to_string(&v.len())?)
}


fn prepare_array_op(matches: ArgMatches, command_name: &str) -> Result<(String, String, parsers::Value), Box<Error>> {
    let submatches = matches.subcommand_matches(command_name).unwrap();
    let predicate_str: &str = submatches.value_of("predicate").unwrap();
    let predicate_str = String::from(predicate_str);


    Ok(parsers::parse_args_str(predicate_str))
}

fn prepare_array_data() -> Result<serde_json::Value, Box<Error>> {
    let buff: String = read_stdin()?;
    let v: serde_json::Value = serde_json::from_str(&buff)?;

    if !v.is_array() {
        return Err(From::from("Expected Array"));
    }

    Ok(v)
}

fn prepare_object_data() -> Result<serde_json::Value, Box<Error>> {
    let buff: String = read_stdin()?;
    let v: serde_json::Value = serde_json::from_str(&buff)?;

    if !v.is_object() {
        return Err(From::from("Expected Array"));
    }

    Ok(v)
}

fn compare<T: PartialEq>(a: T, b: T, operator: &String) -> bool {
    match operator.as_ref() {
        "==" => operators::eq(a, b),
        "!=" => operators::ne(a, b),
        _ => false
    }
}

fn cast_path(elt: &serde_json::Value, path: &String) -> serde_json::Value {
    let mut val = json!(elt);

    if path != "$self" {
        val = json!(elt.pointer(path).unwrap());
    }

    val
}

fn read(matches: ArgMatches) -> Result<(), Box<Error>> {
    let submatches = matches.subcommand_matches("read").unwrap();

    // Use the struct like normal
    let filename = match submatches.value_of("input") {
        Some(s) => String::from(s),
        None => String::new()
    };
    let contents = read_file(filename)?;

    io::stdout().write(contents.as_bytes())?;
    Ok(())
}

fn read_file(filename: String) -> Result<String, Box<Error>> {
    let mut f = File::open(filename)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;

    Ok(s)
}
