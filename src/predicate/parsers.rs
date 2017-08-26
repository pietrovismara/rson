use std::f64;
use std::str::FromStr;

pub enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    None
}

//
// pub fn parse_keys(keys: Vec<&str>) -> Vec<String> {
//     let mut res: Vec<String> = Vec::new();
//     for key in &keys {
//         res.push(parse_key(key));
//     }
//
//     res
// }

pub fn parse_key(key: &str) -> String {
    if key == "$self" {
        return String::from(key);
    }

    let s: Vec<String> = key.split(".").map(|s| String::from(s)).collect();
    let mut res = String::new();
    for c in &s {
        res.push('/');
        res.push_str(c);
    }

    res
}


pub fn parse_args_str(pred_str: String) -> (String, String, Value) {
    let splits: Vec<&str> = pred_str.split(",").collect();
    let val: Vec<&str> = splits[2].split(":").collect();
    // TODO this function should be able to return a String or an f64 depending on the input
    let val = match val[0] {
        "n" => Value::Num(f64::from_str(val[1]).unwrap()),
        "s" => Value::Str(String::from(val[1])),
        "b" => Value::Bool(bool::from_str(val[1]).unwrap()),
        _ => Value::None
    };

    (
        parse_key(splits[0]),
        String::from(splits[1]),
        val
    )
}
