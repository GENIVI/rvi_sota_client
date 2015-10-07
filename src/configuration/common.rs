use toml;
use std::result;
use std::fmt;

pub type Result<T> = result::Result<T, String>;

pub trait ConfTreeParser<C> {
    fn parse(tree: &toml::Table) -> Result<C>;
}

pub fn get_required_key<D>(subtree: &toml::Value, key: &str, group: &str)
    -> Result<D> where D: ParseTomlValue {
    let value = try!(subtree.lookup(key)
                     .ok_or(format!("Missing required key \"{}\" in \"{}\"",
                                    key, group)));
    ParseTomlValue::parse(value, key, group)
}

// This basically does a Option<Result> -> Result<Option> translation
pub fn get_optional_key<D>(subtree: &toml::Value, key: &str, group: &str)
    -> Result<Option<D>> where D: ParseTomlValue {
    match subtree.lookup(key) {
        Some(val) => {
            Ok(Some(try!(ParseTomlValue::parse(val, key, group))))
        },
        None => Ok(None)
    }
}

pub trait ParseTomlValue {
    fn parse(val: &toml::Value, key: &str, group: &str) -> Result<Self>;
}

impl ParseTomlValue for String {
    fn parse(val: &toml::Value, key: &str, group: &str)
        -> Result<String> {
        val.as_str().map(|s| s.to_string())
            .ok_or(format!("Key \"{}\" in \"{}\" is not a string", key, group))
    }
}

impl ParseTomlValue for i32 {
    fn parse(val: &toml::Value, key: &str, group: &str)
        -> Result<i32> {
        val.as_integer().map(|i| i as i32)
            .ok_or(format!("Key \"{}\" in \"{}\" is not a integer", key, group))
    }
}

#[cfg(not(test))]
pub fn format_parser_error(parser: &toml::Parser) -> String {
    let linecol = parser.to_linecol(0);
    format!("parse error: {}:{}: {:?}", linecol.0, linecol.1, parser.errors)
}

#[cfg(test)]
pub fn format_parser_error(parser: &toml::Parser) -> String {
    format!("parse error: {:?}", parser.errors)
}

pub fn stringify<T>(e: T) -> String
    where T: fmt::Display {
    format!("{}", e)
}

#[cfg(test)]
pub fn read_tree(tree: &str) -> Result<toml::Table> {
    let mut parser = toml::Parser::new(tree);
    parser.parse().ok_or(format_parser_error(&parser))
}
