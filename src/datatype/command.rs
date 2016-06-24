use std::str;
use std::str::FromStr;

use nom::{IResult, space, eof};
use datatype::{ClientCredentials, ClientId, ClientSecret, Error, UpdateRequestId};
use datatype::report::{UpdateReport, InstalledSoftware};


#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub enum Command {
    AcceptUpdates(Vec<UpdateRequestId>),
    UpdateReport(UpdateReport),
    Authenticate(Option<ClientCredentials>),
    GetPendingUpdates,
    ListInstalledPackages,
    Shutdown,
    UpdateInstalledPackages,
    ReportInstalledSoftware(InstalledSoftware),
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
        alt_complete!(tag!("AcceptUpdate") | tag!("acc"))
            => { |_| Command::AcceptUpdates(Vec::new()) }
        | alt_complete!(tag!("Authenticate") | tag!("auth"))
            => { |_| Command::Authenticate(None) }
        | alt_complete!(tag!("GetPendingUpdates") | tag!("pen"))
            => { |_| Command::GetPendingUpdates }
        | alt_complete!(tag!("ListInstalledPackages") | tag!("ls"))
            => { |_| Command::ListInstalledPackages }
        | alt_complete!(tag!("Shutdown") | tag!("shutdown"))
            => { |_| Command::Shutdown }
        | alt_complete!(tag!("UpdateInstalledPackages") | tag!("up"))
            => { |_| Command::UpdateInstalledPackages }
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
        Command::AcceptUpdates(_) => match args.len() {
            0 => Err(Error::Command("usage: acc [<id>]".to_string())),
            _ => Ok(Command::AcceptUpdates(args.iter().map(|arg| String::from(*arg)).collect())),
        },

        Command::Authenticate(_) => match args.len() {
            0 => Ok(Command::Authenticate(None)),
            1 => Err(Error::Command("usage: auth <id> <secret>".to_string())),
            2 => Ok(Command::Authenticate(Some(ClientCredentials {
                    id:     ClientId(args[0].to_string()),
                    secret: ClientSecret(args[1].to_string())}))),
            _ => Err(Error::Command(format!("unexpected auth args: {:?}", args))),
        },

        Command::GetPendingUpdates => match args.len() {
            0 => Ok(Command::GetPendingUpdates),
            _ => Err(Error::Command(format!("unexpected pen args: {:?}", args))),
        },

        Command::ListInstalledPackages => match args.len() {
            0 => Ok(Command::ListInstalledPackages),
            _ => Err(Error::Command(format!("unexpected ls args: {:?}", args))),
        },

        Command::Shutdown => match args.len() {
            0 => Ok(Command::Shutdown),
            _ => Err(Error::Command(format!("unexpected shutdown args: {:?}", args))),
        },

        Command::UpdateInstalledPackages => match args.len() {
            0 => Ok(Command::UpdateInstalledPackages),
            _ => Err(Error::Command(format!("unexpected up args: {:?}", args))),
        },

        Command::ReportInstalledSoftware(_) => match args.len() {
            // TODO: Implement feature
            0 => Ok(Command::UpdateInstalledPackages),
            _ => Err(Error::Command(format!("unexpected up args: {:?}", args))),
        },

        Command::UpdateReport(_) => match args.len() {
            // TODO: Implement feature
            0 => Ok(Command::UpdateInstalledPackages),
            _ => Err(Error::Command(format!("unexpected up args: {:?}", args))),
        },
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
                   IResult::Done(&b""[..], (Command::AcceptUpdates(Vec::new()), vec!["1"])));
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
        assert_eq!("acc 1".parse::<Command>().unwrap(), Command::AcceptUpdates(vec!["1".to_string()]));
        assert_eq!("AcceptUpdate this".parse::<Command>().unwrap(), Command::AcceptUpdates(vec!["this".to_string()]));
        assert_eq!("acc some more".parse::<Command>().unwrap(), Command::AcceptUpdates(vec!["some".to_string(), "more".to_string()]));
        assert!("acc".parse::<Command>().is_err());
    }

    #[test]
    fn authenticate_test() {
        assert_eq!("auth".parse::<Command>().unwrap(), Command::Authenticate(None));
        assert_eq!("Authenticate".parse::<Command>().unwrap(), Command::Authenticate(None));
        assert_eq!("auth user pass".parse::<Command>().unwrap(),
                   Command::Authenticate(Some(ClientCredentials {
                       id:     ClientId("user".to_string()),
                       secret: ClientSecret("pass".to_string()),
                   })));
        assert!("auth one".parse::<Command>().is_err());
        assert!("auth one two three".parse::<Command>().is_err());
    }

    #[test]
    fn get_pending_updates_test() {
        assert_eq!("pen".parse::<Command>().unwrap(), Command::GetPendingUpdates);
        assert_eq!("GetPendingUpdates".parse::<Command>().unwrap(), Command::GetPendingUpdates);
        assert!("pen some".parse::<Command>().is_err());
    }

    #[test]
    fn list_installed_test() {
        assert_eq!("ls".parse::<Command>().unwrap(), Command::ListInstalledPackages);
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
        assert_eq!("UpdateInstalledPackages".parse::<Command>().unwrap(), Command::UpdateInstalledPackages);
        assert!("up down".parse::<Command>().is_err());
    }
}
