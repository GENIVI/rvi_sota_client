use rustc_serialize::{Encodable};
use std::str::FromStr;

use datatype::UpdateRequestId;


#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug)]
pub enum Command {
    // UI
    GetPendingUpdates,
    AcceptUpdate(UpdateRequestId),

    PostInstalledPackages,
    ListInstalledPackages,

    Shutdown
}

impl FromStr for Command {

    type Err = ();

    fn from_str(s: &str) -> Result<Command, ()> {
        match s {
            "GetPendingUpdates"     => Ok(Command::GetPendingUpdates),
            "PostInstalledPackages" => Ok(Command::PostInstalledPackages),
            "ListInstalledPackages" => Ok(Command::ListInstalledPackages),
            _                       => Err(()),
        }
    }

}
