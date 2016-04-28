use rustc_serialize::{Encodable};
use std::str::FromStr;

use datatype::{ClientCredentials, UpdateRequestId};


#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug)]
pub enum Command {

    Authenticate(Option<ClientCredentials>),

    // UI
    GetPendingUpdates,
    AcceptUpdate(UpdateRequestId),

    UpdateInstalledPackages,
    ListInstalledPackages,

    Shutdown
}

impl FromStr for Command {

    type Err = ();

    fn from_str(s: &str) -> Result<Command, ()> {
        match s {
            "GetPendingUpdates"     => Ok(Command::GetPendingUpdates),
            "UpdateInstalledPackages" => Ok(Command::UpdateInstalledPackages),
            "ListInstalledPackages" => Ok(Command::ListInstalledPackages),
            _                       => Err(()),
        }
    }

}
