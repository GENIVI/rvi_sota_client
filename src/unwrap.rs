use std::collections::hash_map::Entry;

pub trait Unpack<'a, V> {
    /// Either run the provided closure or print an error message
    fn unpack_or_println<F>(self, run: F, id: u64) -> bool
        where F : Fn(&mut V) -> bool;
}

impl<'a, K, V> Unpack<'a, V> for Entry<'a, K, V> {
    fn unpack_or_println<F>(self, run: F, id: u64) -> bool
        where F : Fn(&mut V) -> bool {
        match self {
            Entry::Occupied(mut entry) => {
                run(entry.get_mut())
            },
            Entry::Vacant(_) => {
                println!("Dropping message with unknown id: {}", id);
                false
            }
        }
    }
}
