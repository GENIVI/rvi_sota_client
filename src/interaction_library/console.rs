use std::io;

use super::gateway::Gateway;
use super::parse::Parse;
use super::print::Print;


pub struct Console;

impl<C, E> Gateway<C, E> for Console
    where
    C: Parse + Send + 'static, E: Print + Send + 'static {

    fn new() -> Console {
        Console
    }

    fn get_line(&self) -> String {

        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);

        return input

    }

    fn put_line(&self, s: String) {
        println!("{}", s);
    }

    fn parse(s: String) -> Option<C> {
        Parse::parse(s)
    }

    fn pretty_print(e: E) -> String {
        e.pretty_print()
    }

}
