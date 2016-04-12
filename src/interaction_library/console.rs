use std::io;
use std::str::FromStr;
use std::string::ToString;

use super::gateway::Gateway;


pub struct Console;

impl<C, E> Gateway<C, E> for Console
    where
    C: FromStr + Send + 'static, E: ToString + Send + 'static {

    fn new() -> Console {
        Console
    }

    fn get_line(&self) -> String {

        print!("> ");
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);

        return input

    }

    fn put_line(&self, s: String) {
        println!("{}", s);
    }

    fn parse(s: String) -> Option<C> {
        s.parse().ok()
    }

    fn pretty_print(e: E) -> String {
        e.to_string()
    }

}
