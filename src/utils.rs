use js_sys::RegExp;
use serde_json::Value;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn parse_int(value: &serde_json::Value) -> Option<i64> {
    match value {
        Value::Number(_) => Some(value.as_i64().unwrap()),
        Value::String(_) => {
            match value.as_str().unwrap().parse::<i64>() {
                Ok(i) => Some(i),
                Err(_) => None
            }
        }
        _ => None
    }
}

pub fn parse_bool(value: &serde_json::Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(_) => value.as_bool().unwrap(),
        Value::Number(_) => value.as_i64().unwrap() != 0,
        Value::String(_) => !value.as_str().unwrap().is_empty(),
        _ => false
    }
}

pub fn regex_replace(src: &str, regex_expr: &str, new: &str, flags: &str) -> String {
    let mut ret = String::from(src);
    let regex = RegExp::new(regex_expr, flags);
    let mut results = regex.exec(src);

    while results.is_some() {
        let mut v = vec![];
        results.unwrap().for_each(&mut |x, _, _| v.push(x));
        for expr in v {
            let target = expr.as_string().unwrap();
            ret = ret.replace(target.as_str(), new);
        }
        results = regex.exec(src);
    }
    ret
}
