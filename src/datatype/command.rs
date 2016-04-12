use rustc_serialize::{Encodable};

use datatype::UpdateRequestId;
use interaction_library::Parse;


#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug)]
pub enum Command {
    // UI
    GetPendingUpdates,
    AcceptUpdate(UpdateRequestId),

    PostInstalledPackages,
    ListInstalledPackages
}

impl Parse for Command {

    fn parse(s: String) -> Option<Command> {
        match s.as_str() {
            "GetPendingUpdates"     => Some(Command::GetPendingUpdates),
            "PostInstalledPackages" => Some(Command::PostInstalledPackages),
            "ListInstalledPackages" => Some(Command::ListInstalledPackages),
            _                       => None,
        }
    }

}
