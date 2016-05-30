use std::str;
use std::str::FromStr;

use nom::{IResult, space, eof};
use datatype::{ClientCredentials, ClientId, ClientSecret, Error, UpdateRequestId};


#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub enum Command {
    AcceptUpdate(UpdateRequestId),
    Authenticate(Option<ClientCredentials>),
    GetPendingUpdates,
    ListInstalledPackages,
    Shutdown,
    UpdateInstalledPackages,
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Command, Error> {
        match command(s.as_bytes()) {
            IResult::Done(_, cmd) => parse_arguments(cmd.0, cmd.1.clone()),
            _ => Err(Error::Command(format!("bad command: {}", s))),
        }
    }
}

named!(command <(Command, Vec<&str>)>, chain!(
    space?
    ~ cmd: alt!(
        alt_complete!(tag!("Authenticate")
                      | tag!("authenticate")
                      | tag!("auth")
        ) => { |_| Command::Authenticate(None) }

        | alt_complete!(tag!("GetPendingUpdates")
                        | tag!("getPendingUpdates")
                        | tag!("pen")
        ) => { |_| Command::GetPendingUpdates }

        | alt_complete!(tag!("AcceptUpdate")
                        | tag!("acceptUpdate")
                        | tag!("acc")
        ) => { |_| Command::AcceptUpdate("".to_owned()) }

        | alt_complete!(tag!("ListInstalledPackages")
                        | tag!("listInstalledPackages")
                        | tag!("ls")
        ) => { |_| Command::ListInstalledPackages }

        | alt_complete!(tag!("Shutdown")
                        | tag!("shutdown")
        ) => { |_| Command::Shutdown }

        | alt_complete!(tag!("UpdateInstalledPackages")
                        | tag!("updateInstalledPackages")
                        | tag!("up")
        ) => { |_| Command::UpdateInstalledPackages }
    )
        ~ args: arguments
        ~ alt!(eof | tag!("\r") | tag!("\n") | tag!(";")),
    move || { (cmd, args) }
));

named!(arguments <&[u8], Vec<&str> >, chain!(
    args: many0!(chain!(
        space?
        ~ text: map_res!(is_not!(" \t\r\n;"), str::from_utf8)
        ~ space?,
        || { text }
    )),
    move || {
        args.into_iter()
            .filter(|arg| arg.len() > 0)
            .collect()
    }
));

fn parse_arguments(cmd: Command, args: Vec<&str>) -> Result<Command, Error> {
    match cmd {
        Command::AcceptUpdate(_) => {
            match args.len() {
                0 => Err(Error::Command("usage: acc <id> <pass>".to_owned())),
                1 => Ok(Command::AcceptUpdate(args[0].to_owned())),
                _ => Err(Error::Command(format!("unexpected acc args: {:?}", args))),
            }
        }

        Command::Authenticate(_) => {
            match args.len() {
                0 => Ok(Command::Authenticate(None)),
                1 => Err(Error::Command("usage: auth <user> <pass>".to_owned())),
                2 => {
                    let (user, pass) = (args[0].to_owned(), args[1].to_owned());
                    Ok(Command::Authenticate(Some(ClientCredentials {
                        id: ClientId { get: user },
                        secret: ClientSecret { get: pass },
                    })))
                }
                _ => Err(Error::Command(format!("unexpected auth args: {:?}", args))),
            }
        }

        Command::GetPendingUpdates => {
            match args.len() {
                0 => Ok(Command::GetPendingUpdates),
                _ => Err(Error::Command(format!("unexpected pen args: {:?}", args))),
            }
        }

        Command::ListInstalledPackages => {
            match args.len() {
                0 => Ok(Command::ListInstalledPackages),
                _ => Err(Error::Command(format!("unexpected ls args: {:?}", args))),
            }
        }

        Command::Shutdown => {
            match args.len() {
                0 => Ok(Command::Shutdown),
                _ => Err(Error::Command(format!("unexpected shutdown args: {:?}", args))),
            }
        }

        Command::UpdateInstalledPackages => {
            match args.len() {
                0 => Ok(Command::UpdateInstalledPackages),
                _ => Err(Error::Command(format!("unexpected up args: {:?}", args))),
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{command, arguments};
    use datatype::{Command, ClientCredentials, ClientId, ClientSecret};
    use nom::IResult;

    #[test]
    fn parse_command_test() {
        assert_eq!(command(&b"auth foo bar"[..]),
                   IResult::Done(&b""[..], (Command::Authenticate(None), vec!["foo", "bar"])));
        assert_eq!(command(&b"acc 1"[..]),
                   IResult::Done(&b""[..], (Command::AcceptUpdate("".to_owned()), vec!["1"])));
        assert_eq!(command(&b"ls;\n"[..]),
                   IResult::Done(&b"\n"[..], (Command::ListInstalledPackages, Vec::new())));
    }

    #[test]
    fn parse_arguments_test() {
        assert_eq!(arguments(&b"one"[..]), IResult::Done(&b""[..], vec!["one"]));
        assert_eq!(arguments(&b"foo bar"[..]), IResult::Done(&b""[..], vec!["foo", "bar"]));
        assert_eq!(arguments(&b"n=5"[..]), IResult::Done(&b""[..], vec!["n=5"]));
        assert_eq!(arguments(&b""[..]), IResult::Done(&b""[..], Vec::new()));
        assert_eq!(arguments(&b" \t some"[..]), IResult::Done(&b""[..], vec!["some"]));
        assert_eq!(arguments(&b";"[..]), IResult::Done(&b";"[..], Vec::new()));
    }

    #[test]
    fn accept_update_test() {
        assert_eq!("acc 1".parse::<Command>().unwrap(), Command::AcceptUpdate("1".to_owned()));
        assert_eq!("acceptUpdate 2".parse::<Command>().unwrap(), Command::AcceptUpdate("2".to_owned()));
        assert_eq!("AcceptUpdate 3".parse::<Command>().unwrap(), Command::AcceptUpdate("3".to_owned()));
        assert_eq!("acc some".parse::<Command>().unwrap(), Command::AcceptUpdate("some".to_owned()));
        assert!("acc".parse::<Command>().is_err());
        assert!("acc more than one".parse::<Command>().is_err());
    }

    #[test]
    fn authenticate_test() {
        assert_eq!("auth".parse::<Command>().unwrap(), Command::Authenticate(None));
        assert_eq!("authenticate".parse::<Command>().unwrap(), Command::Authenticate(None));
        assert_eq!("Authenticate".parse::<Command>().unwrap(), Command::Authenticate(None));
        assert_eq!("auth user pass".parse::<Command>().unwrap(),
                   Command::Authenticate(Some(ClientCredentials {
                       id: ClientId { get: "user".to_owned() },
                       secret: ClientSecret { get: "pass".to_owned() },
                   })));
        assert!("auth one".parse::<Command>().is_err());
        assert!("auth one two three".parse::<Command>().is_err());
    }

    #[test]
    fn get_pending_updates_test() {
        assert_eq!("pen".parse::<Command>().unwrap(), Command::GetPendingUpdates);
        assert_eq!("getPendingUpdates".parse::<Command>().unwrap(), Command::GetPendingUpdates);
        assert_eq!("GetPendingUpdates".parse::<Command>().unwrap(), Command::GetPendingUpdates);
        assert!("pen some".parse::<Command>().is_err());
    }

    #[test]
    fn list_installed_test() {
        assert_eq!("ls".parse::<Command>().unwrap(), Command::ListInstalledPackages);
        assert_eq!("listInstalledPackages".parse::<Command>().unwrap(), Command::ListInstalledPackages);
        assert_eq!("ListInstalledPackages".parse::<Command>().unwrap(), Command::ListInstalledPackages);
        assert!("ls some".parse::<Command>().is_err());
    }

    #[test]
    fn shutdown_test() {
        assert_eq!("shutdown".parse::<Command>().unwrap(), Command::Shutdown);
        assert_eq!("Shutdown".parse::<Command>().unwrap(), Command::Shutdown);
        assert!("shutdown now".parse::<Command>().is_err());
        assert!("Shutdown 1 2".parse::<Command>().is_err());
    }

    #[test]
    fn update_installed_test() {
        assert_eq!("up".parse::<Command>().unwrap(), Command::UpdateInstalledPackages);
        assert_eq!("updateInstalledPackages".parse::<Command>().unwrap(), Command::UpdateInstalledPackages);
        assert_eq!("UpdateInstalledPackages".parse::<Command>().unwrap(), Command::UpdateInstalledPackages);
        assert!("up down".parse::<Command>().is_err());
    }
}
