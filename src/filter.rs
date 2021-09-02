use hlua::{Lua, LuaError};
use serde_json::{Map, Value};
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref LUA_IDENTIFIER_CLEANUP: Regex = Regex::new(r"[^A-Za-z_]").unwrap();
    static ref LUA_STRING_ESCAPE: Regex = Regex::new(r"([\n])").unwrap();
}

pub fn show_log_entry(log_entry: &Map<String, Value>, filter_expr: &str, implicit_return: bool) -> Result<bool, LuaError> {
  let mut lua = Lua::new();
  lua.openlibs();

  let script = object_to_record(log_entry, false);
  println!("{}", script);
  lua.execute::<()>(&script)?;

  if implicit_return {
    lua.execute(&format!("return {};", filter_expr))
  } else {
    lua.execute(filter_expr)
  }
}

fn object_to_record(object: &Map<String, Value>, nested: bool) -> String {

    let lines: Vec<String> = object.iter()
        .map(|(key, value)| {
            let mut script = String::new();
            let key_name = LUA_IDENTIFIER_CLEANUP.replace_all(key, "_");
            script.push_str(&format!("{} = ", key_name));
            match value {
                Value::String(ref string_value) => {
                    script.push_str(&format!("\"{}\"", escape_lua_string(string_value)))
                },
                Value::Bool(ref bool_value) => {
                    script.push_str(&bool_value.to_string())
                }
                Value::Number(ref number_value) => {
                    script.push_str(&number_value.to_string())
                }
                Value::Object(nested_object) => {
                    let object_string = object_to_record(nested_object, true);
                    script.push_str(&format!("{{{}}}", object_string))
                }
                _ => {
                    script.push_str("\"unsupported\"")
                }
            }
            script
        }).collect();
    lines.join(if nested {","} else {"\n"})
}

fn escape_lua_string(src: &str) -> String {
    let mut escaped = String::with_capacity(src.len());
    for c in src.chars() {
        match c {
            '\n' => escaped += "\\n",
            '\r' => escaped += "\\r",
            '\t' => escaped += "\\t",
            '"' => escaped += "\\\"",
            '\'' => escaped += "\\'",
            '[' => escaped += "\\[",
            ']' => escaped += "\\]",
            '\\' => escaped += "\\\\",
            c => escaped += &format!("{}", c),
        }
    }
    escaped
}

/*
#[cfg(test)]
mod tests {
  use super::*;

  use maplit::btreemap;

  fn test_log_entry() -> BTreeMap<String, String> {
    btreemap! {"message".to_string() => "something happend".to_string(),
    "time".to_string() => "2017-07-06T15:21:16".to_string(),
    "process".to_string() => "rust".to_string(),
    "fu".to_string() => "bower".to_string(),
    "level".to_string() => "info".to_string()}
  }

  #[test]
  fn allow_all() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(true, show_log_entry(&log_entry, "true", true).unwrap());
  }

  #[test]
  fn deny_all() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(false, show_log_entry(&log_entry, "false", true).unwrap());
  }

  #[test]
  fn filter_process() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(true, show_log_entry(&log_entry, r#"process == "rust""#, true).unwrap());
    assert_eq!(false, show_log_entry(&log_entry, r#"process == "meep""#, true).unwrap());
  }

  #[test]
  fn filter_logical_operators() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(true, show_log_entry(&log_entry, r#"process == "rust" and fu == "bower""#, true).unwrap());
    assert_eq!(true, show_log_entry(&log_entry, r#"process == "rust" or fu == "bauer""#, true).unwrap());
  }

  #[test]
  fn filter_contains() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(true, show_log_entry(&log_entry, r#"string.find(message, "something") ~= nil"#, true).unwrap());
    assert_eq!(false, show_log_entry(&log_entry, r#"string.find(message, "bla") ~= nil"#, true).unwrap());
  }

  #[test]
  fn filter_regex() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(true, show_log_entry(&log_entry, r#"string.find(fu, "bow.*") ~= nil"#, true).unwrap());
    assert_eq!(false, show_log_entry(&log_entry, r#"string.find(fu, "bow.*sd") ~= nil"#, true).unwrap());
  }

  #[test]
  fn unknown_variable() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(
      false,
      show_log_entry(&log_entry, r#"sdkfjsdfjsf ~= nil and string.find(sdkfjsdfjsf, "bow.*") ~= nil"#, true).unwrap()
    );
  }

  #[test]
  fn no_implicit_return() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(
      true,
      show_log_entry(&log_entry, r#"if 3 > 2 then return true else return false end"#, false).unwrap()
    );
    assert_eq!(
      false,
      show_log_entry(&log_entry, r#"if 1 > 2 then return true else return false end"#, false).unwrap()
    );
  }
}
*/
