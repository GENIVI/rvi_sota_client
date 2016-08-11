use std::fmt;
use std::str;
use std::str::FromStr;

use nom::{IResult, space, eof};
use datatype::{ClientCredentials, ClientId, ClientSecret, Error, InstalledSoftware,
               UpdateReport, UpdateRequestId};


#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub enum Command {
    Authenticate(Option<ClientCredentials>),
    Shutdown,

    GetPendingUpdates,
    AcceptUpdates(Vec<UpdateRequestId>),
    AbortUpdates(Vec<UpdateRequestId>),

    ListInstalledPackages,
    UpdateInstalledPackages,

    SendInstalledSoftware(Option<InstalledSoftware>),
    SendSystemInfo,
    SendUpdateReport(Option<UpdateReport>),
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Command, Error> {
        match command(s.as_bytes()) {
            IResult::Done(_, cmd) => parse_arguments(cmd.0, cmd.1.clone()),
            _                     => Err(Error::Command(format!("bad command: {}", s)))
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


named!(command <(Command, Vec<&str>)>, chain!(
    space?
    ~ cmd: alt!(
        alt_complete!(tag!("AcceptUpdates") | tag!("acc"))
            => { |_| Command::AcceptUpdates(Vec::new()) }
        | alt_complete!(tag!("AbortUpdates") | tag!("abort"))
            => { |_| Command::AbortUpdates(Vec::new()) }
        | alt_complete!(tag!("Authenticate") | tag!("auth"))
            => { |_| Command::Authenticate(None) }
        | alt_complete!(tag!("GetPendingUpdates") | tag!("pen"))
            => { |_| Command::GetPendingUpdates }
        | alt_complete!(tag!("ListInstalledPackages") | tag!("ls"))
            => { |_| Command::ListInstalledPackages }
        | alt_complete!(tag!("SendInstalledSoftware") | tag!("sendinst"))
            => { |_| Command::SendInstalledSoftware(None) }
        | alt_complete!(tag!("SendSystemInfo") | tag!("info"))
            => { |_| Command::SendSystemInfo }
        | alt_complete!(tag!("SendUpdateReport") | tag!("sendup"))
            => { |_| Command::SendUpdateReport(None) }
        | alt_complete!(tag!("Shutdown") | tag!("shutdown"))
            => { |_| Command::Shutdown }
        | alt_complete!(tag!("UpdateInstalledPackages") | tag!("upinst"))
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

        Command::AbortUpdates(_) => match args.len() {
            0 => Err(Error::Command("usage: abort [<id>]".to_string())),
            _ => Ok(Command::AbortUpdates(args.iter().map(|arg| String::from(*arg)).collect())),
        },

        Command::Authenticate(_) => match args.len() {
            0 => Ok(Command::Authenticate(None)),
            1 => Err(Error::Command("usage: auth <id> <secret>".to_string())),
            2 => Ok(Command::Authenticate(Some(ClientCredentials {
                    id:     ClientId(args[0].to_string()),
                    secret: ClientSecret(args[1].to_string())}))),
            _ => Err(Error::Command(format!("unexpected Authenticate args: {:?}", args))),
        },

        Command::GetPendingUpdates => match args.len() {
            0 => Ok(Command::GetPendingUpdates),
            _ => Err(Error::Command(format!("unexpected GetPendingUpdates args: {:?}", args))),
        },

        Command::ListInstalledPackages => match args.len() {
            0 => Ok(Command::ListInstalledPackages),
            _ => Err(Error::Command(format!("unexpected ListInstalledPackages args: {:?}", args))),
        },

        Command::SendInstalledSoftware(_) => match args.len() {
            0 => Ok(Command::SendInstalledSoftware(None)),
            _ => Err(Error::Command(format!("unexpected SendInstalledSoftware args: {:?}", args))),
        },

        Command::SendUpdateReport(_) => match args.len() {
            0 => Ok(Command::SendUpdateReport(None)),
            _ => Err(Error::Command(format!("unexpected SendUpdateReport args: {:?}", args))),
        },

        Command::Shutdown => match args.len() {
            0 => Ok(Command::Shutdown),
            _ => Err(Error::Command(format!("unexpected Shutdown args: {:?}", args))),
        },

        Command::SendSystemInfo => match args.len() {
            0 => Ok(Command::SendSystemInfo),
            _ => Err(Error::Command(format!("unexpected SendSystemInfo args: {:?}", args))),
        },

        Command::UpdateInstalledPackages => match args.len() {
            0 => Ok(Command::UpdateInstalledPackages),
            _ => Err(Error::Command(format!("unexpected UpdateInstalledPackages args: {:?}", args))),
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
    fn accept_updates_test() {
        assert_eq!("acc 1".parse::<Command>().unwrap(), Command::AcceptUpdates(vec!["1".to_string()]));
        assert_eq!("AcceptUpdates this".parse::<Command>().unwrap(), Command::AcceptUpdates(vec!["this".to_string()]));
        assert_eq!("acc some more".parse::<Command>().unwrap(), Command::AcceptUpdates(vec!["some".to_string(), "more".to_string()]));
        assert!("acc".parse::<Command>().is_err());
    }

    #[test]
    fn abort_updates_test() {
        assert_eq!("abort 1".parse::<Command>().unwrap(), Command::AbortUpdates(vec!["1".to_string()]));
        assert_eq!("AbortUpdates this".parse::<Command>().unwrap(), Command::AbortUpdates(vec!["this".to_string()]));
        assert_eq!("abort some more".parse::<Command>().unwrap(), Command::AbortUpdates(vec!["some".to_string(), "more".to_string()]));
        assert!("abort".parse::<Command>().is_err());
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
    fn send_installed_software_test() {
        assert_eq!("sendinst".parse::<Command>().unwrap(), Command::SendInstalledSoftware(None));
        assert_eq!("SendInstalledSoftware".parse::<Command>().unwrap(), Command::SendInstalledSoftware(None));
        assert!("sendinst some".parse::<Command>().is_err());
    }

    #[test]
    fn send_update_report_test() {
        assert_eq!("sendup".parse::<Command>().unwrap(), Command::SendUpdateReport(None));
        assert_eq!("SendUpdateReport".parse::<Command>().unwrap(), Command::SendUpdateReport(None));
        assert!("sendup some".parse::<Command>().is_err());
    }

    #[test]
    fn shutdown_test() {
        assert_eq!("shutdown".parse::<Command>().unwrap(), Command::Shutdown);
        assert_eq!("Shutdown".parse::<Command>().unwrap(), Command::Shutdown);
        assert!("shutdown now".parse::<Command>().is_err());
        assert!("Shutdown 1 2".parse::<Command>().is_err());
    }

    #[test]
    fn sendsysteminfo_test() {
        assert_eq!("info".parse::<Command>().unwrap(), Command::SendSystemInfo);
        assert_eq!("SendSystemInfo".parse::<Command>().unwrap(), Command::SendSystemInfo);
        assert!("info please".parse::<Command>().is_err());
        assert!("SendSystemInfo 1 2".parse::<Command>().is_err());
    }

    #[test]
    fn update_installed_test() {
        assert_eq!("upinst".parse::<Command>().unwrap(), Command::UpdateInstalledPackages);
        assert_eq!("UpdateInstalledPackages".parse::<Command>().unwrap(), Command::UpdateInstalledPackages);
        assert!("upinst down".parse::<Command>().is_err());
    }
}
