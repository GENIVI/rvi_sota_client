use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str;
use std::str::FromStr;

use nom::{IResult, space, eof};
use datatype::{ClientCredentials, ClientId, ClientSecret, DownloadComplete, Error,
               InstalledSoftware, Package, UpdateReport, UpdateRequestId,
               UpdateResultCode};


/// System-wide commands that are sent to the interpreter.
#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug, Clone)]
pub enum Command {
    /// Authenticate with the auth server.
    Authenticate(Option<ClientCredentials>),
    /// Shutdown the client immediately.
    Shutdown,

    /// Check for any new updates.
    GetNewUpdates,
    /// List the installed packages on the system.
    ListInstalledPackages,
    /// Get the latest system information, and optionally publish it to Core.
    RefreshSystemInfo(bool),

    /// Start downloading one or more updates.
    StartDownload(Vec<UpdateRequestId>),
    /// Start installing an update
    StartInstall(DownloadComplete),

    /// Send a list of packages to the Core server.
    SendInstalledPackages(Vec<Package>),
    /// Send a list of all packages and firmware to the Core server.
    SendInstalledSoftware(InstalledSoftware),
    /// Send a package update report to the Core server.
    SendUpdateReport(UpdateReport),
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self)
    }
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


named!(command <(Command, Vec<&str>)>, chain!(
    space?
    ~ cmd: alt!(
        alt_complete!(tag!("Authenticate") | tag!("auth"))
            => { |_| Command::Authenticate(None) }
        | alt_complete!(tag!("GetNewUpdates") | tag!("new"))
            => { |_| Command::GetNewUpdates }
        | alt_complete!(tag!("ListInstalledPackages") | tag!("ls"))
            => { |_| Command::ListInstalledPackages }
        | alt_complete!(tag!("RefreshSystemInfo") | tag!("info"))
            => { |_| Command::RefreshSystemInfo(false) }
        | alt_complete!(tag!("Shutdown") | tag!("shutdown"))
            => { |_| Command::Shutdown }
        | alt_complete!(tag!("SendInstalledPackages") | tag!("sendpack"))
            => { |_| Command::SendInstalledPackages(Vec::new()) }
        | alt_complete!(tag!("SendInstalledSoftware") | tag!("sendinst"))
            => { |_| Command::SendInstalledSoftware(InstalledSoftware::default()) }
        | alt_complete!(tag!("SendUpdateReport") | tag!("sendup"))
            => { |_| Command::SendUpdateReport(UpdateReport::default()) }
        | alt_complete!(tag!("StartDownload") | tag!("dl"))
            => { |_| Command::StartDownload(Vec::new()) }
        | alt_complete!(tag!("StartInstall") | tag!("inst"))
            => { |_| Command::StartInstall(DownloadComplete::default()) }
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
        Command::Authenticate(_) => match args.len() {
            0 => Ok(Command::Authenticate(None)),
            1 => Err(Error::Command("usage: auth <client-id> <client-secret>".to_string())),
            2 => Ok(Command::Authenticate(Some(ClientCredentials {
                    client_id:     ClientId(args[0].to_string()),
                    client_secret: ClientSecret(args[1].to_string())}))),
            _ => Err(Error::Command(format!("unexpected Authenticate args: {:?}", args))),
        },

        Command::GetNewUpdates => match args.len() {
            0 => Ok(Command::GetNewUpdates),
            _ => Err(Error::Command(format!("unexpected GetNewUpdates args: {:?}", args))),
        },

        Command::ListInstalledPackages => match args.len() {
            0 => Ok(Command::ListInstalledPackages),
            _ => Err(Error::Command(format!("unexpected ListInstalledPackages args: {:?}", args))),
        },

        Command::RefreshSystemInfo(_) => match args.len() {
            0 => Ok(Command::RefreshSystemInfo(false)),
            1 => {
                if let Ok(b) = args[0].parse::<bool>() {
                    Ok(Command::RefreshSystemInfo(b))
                } else {
                    Err(Error::Command("couldn't parse 1st argument as boolean".to_string()))
                }
            }
            _ => Err(Error::Command(format!("unexpected RefreshSystemInfo args: {:?}", args))),
        },

        Command::SendInstalledPackages(_) => match args.len() {
            0 | 1 => Err(Error::Command("usage: sendpack (<name> <version> )+".to_string())),
            n if n % 2 == 0 => {
                let (names, versions): (Vec<(_, &str)>, Vec<(_, &str)>) =
                    args.into_iter().enumerate().partition(|&(n, _)| n % 2 == 0);
                let packages = names.into_iter().zip(versions.into_iter())
                    .map(|((_, name), (_, version))| Package {
                        name:    name.to_string(),
                        version: version.to_string()
                    }).collect::<Vec<Package>>();
                Ok(Command::SendInstalledPackages(packages))
            }
            _ => Err(Error::Command(format!("SendInstalledPackages expects an even number of 'name version' pairs"))),
        },

        Command::SendInstalledSoftware(_) => match args.len() {
            // FIXME(PRO-1160): args
            _ => Err(Error::Command(format!("unexpected SendInstalledSoftware args: {:?}", args))),
        },

        Command::SendUpdateReport(_) => match args.len() {
            0 | 1 | 2 => Err(Error::Command("usage: sendup <update-id> <result-code> <result-text>".to_string())),
            3 => {
                if let Ok(code) = args[1].parse::<UpdateResultCode>() {
                    let id   = args[0].to_string();
                    let text = args[2].to_string();
                    Ok(Command::SendUpdateReport(UpdateReport::single(id, code, text)))
                } else {
                    Err(Error::Command("couldn't parse 2nd argument as an UpdateResultCode".to_string()))
                }
            }
            _ => Err(Error::Command(format!("unexpected SendUpdateReport args: {:?}", args))),
        },

        Command::Shutdown => match args.len() {
            0 => Ok(Command::Shutdown),
            _ => Err(Error::Command(format!("unexpected Shutdown args: {:?}", args))),
        },

        Command::StartDownload(_) => match args.len() {
            0 => Err(Error::Command("usage: dl [<id>]".to_string())),
            _ => Ok(Command::StartDownload(args.iter().map(|arg| String::from(*arg)).collect())),
        },

        Command::StartInstall(_) => match args.len() {
            // FIXME(PRO-1160): args
            _ => Err(Error::Command(format!("unexpected StartInstall args: {:?}", args))),
        },

    }
}


#[cfg(test)]
mod tests {
    use super::{command, arguments};
    use datatype::{Command, ClientCredentials, ClientId, ClientSecret, Package,
                   UpdateReport, UpdateResultCode};
    use nom::IResult;


    #[test]
    fn parse_command_test() {
        assert_eq!(command(&b"auth foo bar"[..]),
                   IResult::Done(&b""[..], (Command::Authenticate(None), vec!["foo", "bar"])));
        assert_eq!(command(&b"dl 1"[..]),
                   IResult::Done(&b""[..], (Command::StartDownload(Vec::new()), vec!["1"])));
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
    fn authenticate_test() {
        assert_eq!("Authenticate".parse::<Command>().unwrap(), Command::Authenticate(None));
        assert_eq!("auth".parse::<Command>().unwrap(), Command::Authenticate(None));
        assert_eq!("auth user pass".parse::<Command>().unwrap(),
                   Command::Authenticate(Some(ClientCredentials {
                       client_id:     ClientId("user".to_string()),
                       client_secret: ClientSecret("pass".to_string()),
                   })));
        assert!("auth one".parse::<Command>().is_err());
        assert!("auth one two three".parse::<Command>().is_err());
    }

    #[test]
    fn get_new_updates_test() {
        assert_eq!("GetNewUpdates".parse::<Command>().unwrap(), Command::GetNewUpdates);
        assert_eq!("new".parse::<Command>().unwrap(), Command::GetNewUpdates);
        assert!("new old".parse::<Command>().is_err());
    }

    #[test]
    fn list_installed_test() {
        assert_eq!("ListInstalledPackages".parse::<Command>().unwrap(), Command::ListInstalledPackages);
        assert_eq!("ls".parse::<Command>().unwrap(), Command::ListInstalledPackages);
        assert!("ls some".parse::<Command>().is_err());
    }

    #[test]
    fn refresh_system_info_test() {
        assert_eq!("RefreshSystemInfo true".parse::<Command>().unwrap(), Command::RefreshSystemInfo(true));
        assert_eq!("info".parse::<Command>().unwrap(), Command::RefreshSystemInfo(false));
        assert!("RefreshSystemInfo 1 2".parse::<Command>().is_err());
        assert!("info please".parse::<Command>().is_err());
    }

    #[test]
    fn send_installed_packages_test() {
        assert_eq!("SendInstalledPackages myname myversion".parse::<Command>().unwrap(),
                   Command::SendInstalledPackages(vec![Package {
                       name:    "myname".to_string(),
                       version: "myversion".to_string()
                   }]));
        assert_eq!("sendpack n1 v1 n2 v2".parse::<Command>().unwrap(),
                   Command::SendInstalledPackages(vec![Package {
                       name:    "n1".to_string(),
                       version: "v1".to_string()
                   }, Package {
                       name:    "n2".to_string(),
                       version: "v2".to_string()
                   }]));
        assert!("SendInstalledPackages some".parse::<Command>().is_err());
        assert!("sendpack 1 2 3".parse::<Command>().is_err());
    }

    #[test]
    fn send_installed_software_test() {
        assert!("SendInstalledSoftware".parse::<Command>().is_err());
        assert!("sendsoft some".parse::<Command>().is_err());
    }

    #[test]
    fn send_update_report_test() {
        assert_eq!("SendUpdateReport myid OK done".parse::<Command>().unwrap(),
                   Command::SendUpdateReport(UpdateReport::single(
                       "myid".to_string(), UpdateResultCode::OK, "done".to_string()
                   )));
        assert_eq!("sendup myid 19 generr".parse::<Command>().unwrap(),
                   Command::SendUpdateReport(UpdateReport::single(
                       "myid".to_string(), UpdateResultCode::GENERAL_ERROR, "generr".to_string()
                   )));
        assert!("sendup myid 20 nosuch".parse::<Command>().is_err());
        assert!("SendInstalledPackages".parse::<Command>().is_err());
        assert!("sendup 1 2 3 4".parse::<Command>().is_err());
    }

    #[test]
    fn shutdown_test() {
        assert_eq!("Shutdown".parse::<Command>().unwrap(), Command::Shutdown);
        assert_eq!("shutdown".parse::<Command>().unwrap(), Command::Shutdown);
        assert!("Shutdown 1 2".parse::<Command>().is_err());
        assert!("shutdown now".parse::<Command>().is_err());
    }

    #[test]
    fn start_download_test() {
        assert_eq!("StartDownload this".parse::<Command>().unwrap(),
                   Command::StartDownload(vec!["this".to_string()]));
        assert_eq!("dl some more".parse::<Command>().unwrap(),
                   Command::StartDownload(vec!["some".to_string(), "more".to_string()]));
        assert!("dl".parse::<Command>().is_err());
    }

    #[test]
    fn start_install_test() {
        assert!("StartInstall".parse::<Command>().is_err());
        assert!("inst more than one".parse::<Command>().is_err());
    }
}
