use dbus::{FromMessageItem, MessageItem};
use toml::{decode, Table, Value};

/// DBus error string to indicate a missing argument.
static MISSING_ARG: &'static str = "Error.MissingArgument";
/// DBus error string to indicate a malformed argument.
static MALFORMED_ARG: &'static str = "Error.MalformedArgument";

/// Format a DBus error message indicating a missing argument.
pub fn missing_arg() -> (&'static str, String) {
    (MISSING_ARG, "Missing argument".to_string())
}

/// Format a DBus error message indicating a malformed argument.
pub fn malformed_arg() -> (&'static str, String) {
    (MALFORMED_ARG, "Malformed argument".to_string())
}


struct DecodableValue(Value);

impl<'a> FromMessageItem<'a> for DecodableValue {
    fn from(m: &'a MessageItem) -> Result<Self, ()> {
        match m {
            &MessageItem::Str(ref b) => Ok(DecodableValue(Value::String(b.clone()))),
            &MessageItem::Bool(ref b) => Ok(DecodableValue(Value::Boolean(*b))),
            &MessageItem::Byte(ref b) => Ok(DecodableValue(Value::Integer(*b as i64))),
            &MessageItem::Int16(ref b) => Ok(DecodableValue(Value::Integer(*b as i64))),
            &MessageItem::Int32(ref b) => Ok(DecodableValue(Value::Integer(*b as i64))),
            &MessageItem::Int64(ref b) => Ok(DecodableValue(Value::Integer(*b as i64))),
            &MessageItem::UInt16(ref b) => Ok(DecodableValue(Value::Integer(*b as i64))),
            &MessageItem::UInt32(ref b) => Ok(DecodableValue(Value::Integer(*b as i64))),
            &MessageItem::UInt64(ref b) => Ok(DecodableValue(Value::Integer(*b as i64))),
            &MessageItem::Variant(ref b) => FromMessageItem::from(&**b),
            _ => Err(())
        }
    }
}

pub struct DecodableStruct(pub Value);

impl<'a> FromMessageItem<'a> for DecodableStruct {
    fn from(m: &'a MessageItem) -> Result<Self, ()> {
        let arr: &Vec<MessageItem> = try!(FromMessageItem::from(m));
        arr.iter()
            .map(|entry| {
                let v: Result<(&MessageItem, &MessageItem), ()> = FromMessageItem::from(entry);
                v.and_then(|(k, v)| {
                    let k: Result<&String,()> = FromMessageItem::from(k);
                    k.and_then(|k| {
                        let v: Result<DecodableValue,()> = FromMessageItem::from(v);
                        v.map(|v| (k.clone(), v.0)) }) }) })
            .collect::<Result<Vec<(_, _)>, ()>>()
            .map(|arr| DecodableStruct(Value::Table(arr.into_iter().collect::<Table>())))
    }
}


use event::outbound::{OperationResult, OperationResults};

impl<'a> FromMessageItem<'a> for OperationResult {
    fn from(m: &'a MessageItem) -> Result<Self, ()> {
        let m: DecodableStruct = try!(FromMessageItem::from(m));
        decode::<OperationResult>(m.0).ok_or(())
    }
}

impl<'a> FromMessageItem<'a> for OperationResults {
    fn from(m: &'a MessageItem) -> Result<Self, ()> {
        let arr: &Vec<MessageItem> = try!(FromMessageItem::from(m));
        arr.into_iter()
            .map(|i| {
                let i: Result<OperationResult, ()> = FromMessageItem::from(i);
                i })
            .collect::<Result<Vec<_>, ()>>()
            .map(|a| OperationResults(a))
    }
}

use event::outbound::{InstalledPackage, InstalledPackages};

impl<'a> FromMessageItem<'a> for InstalledPackage {
    fn from(m: &'a MessageItem) -> Result<Self, ()> {
        let m: DecodableStruct = try!(FromMessageItem::from(m));
        decode::<InstalledPackage>(m.0).ok_or(())
    }
}

impl<'a> FromMessageItem<'a> for InstalledPackages {
    fn from(m: &'a MessageItem) -> Result<Self, ()> {
        let arr: &Vec<MessageItem> = try!(FromMessageItem::from(m));
        arr.into_iter()
            .map(|i| {
                let i: Result<InstalledPackage, ()> = FromMessageItem::from(i);
                i })
            .collect::<Result<Vec<_>, ()>>()
            .map(|a| InstalledPackages(a))
    }
}

use event::outbound::{InstalledFirmware, InstalledFirmwares};

impl<'a> FromMessageItem<'a> for InstalledFirmware {
    fn from(m: &'a MessageItem) -> Result<Self, ()> {
        let m: DecodableStruct = try!(FromMessageItem::from(m));
        decode::<InstalledFirmware>(m.0).ok_or(())
    }
}

impl<'a> FromMessageItem<'a> for InstalledFirmwares {
    fn from(m: &'a MessageItem) -> Result<Self, ()> {
        let arr: &Vec<MessageItem> = try!(FromMessageItem::from(m));
        arr.into_iter()
            .map(|i| {
                let i: Result<InstalledFirmware, ()> = FromMessageItem::from(i);
                i })
            .collect::<Result<Vec<_>, ()>>()
            .map(|a| InstalledFirmwares(a))
    }
}

