//! Helper functions for working with packages.

use std::fmt;
use dbus::{FromMessageItem, MessageItem};
use std::ops::Deref;

/// Encodes a package, defined through a `name` and `version`.
#[derive(RustcDecodable, RustcEncodable, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PackageId {
    /// The name of the package.
    pub name: String,
    /// The version of the package.
    pub version: String
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.name, self.version)
    }
}

impl<'a> From<&'a PackageId> for MessageItem {
    fn from(p: &PackageId) -> MessageItem {
        let n: &str = &p.name;
        let v: &str = &p.version;
        let name = MessageItem::from(
            (MessageItem::from("name".to_string()),
             MessageItem::from(n)));
        let version = MessageItem::from(
            (MessageItem::from("version".to_string()),
             MessageItem::from(v)));

        MessageItem::new_array(vec!(name, version)).unwrap()
    }
}

impl<'a> FromMessageItem<'a> for PackageId {
    fn from(i: &'a MessageItem) -> Result<Self, ()> {
        let package: &Vec<MessageItem> = match i {
            &MessageItem::Array(ref value, _) => value,
            _ => return Err(())
        };

        let mut name: Option<String> = None;
        let mut version: Option<String> = None;

        for ref entry in package {
            let (key_entry, val_entry) = match *entry {
                &MessageItem::DictEntry(ref key, ref val) =>
                    (key.deref(), val.deref()),
                _ => return Err(())
            };

            let key: &String = try!(FromMessageItem::from(key_entry));
            let val: &String = try!(FromMessageItem::from(val_entry));

            match key.as_ref() {
                "name" => { name = Some(val.clone()); },
                "version" => { version = Some(val.clone()); },
                _ => return Err(())
            }
        }

        Ok(PackageId {
            name: try!(name.ok_or(())),
            version: try!(version.ok_or(()))
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dbus::*;

    #[test]
    fn it_properly_de_encodes_a_package_id_for_dbus() {
        let package = PackageId {
            name: "name".to_string(),
            version: "version".to_string()
        };

        let message_item = MessageItem::from(&package);
        let decoded: PackageId = FromMessageItem::from(&message_item).unwrap();

        assert_eq!(decoded, package);
    }
}
