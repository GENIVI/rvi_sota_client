//! Handles caching and storage on disk for in-progress transfers and the assembly and verification
//! of finished transfers

use std::fs;
use std::fs::{OpenOptions, DirEntry, File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::vec::Vec;
use std::str::FromStr;

use time;

#[cfg(test)] use rand;
#[cfg(test)] use rand::Rng;
#[cfg(test)] use test_library::PathPrefix;

use crypto::sha1::Sha1;
use crypto::digest::Digest;

use rustc_serialize::base64::FromBase64;

use event::UpdateId;

/// Type for storing the metadata of a in-progress transfer, which is defined as one package.
/// Will clear out the chunks on disk when freed.
pub struct Transfer {
    pub update_id: UpdateId,
    /// SHA1 checksum of the fully assembled package.
    pub checksum: String,
    /// `Vector` of transferred chunks.
    pub transferred_chunks: Vec<u64>,
    /// Path to the directory, where chunks will be cached and finished packages will be stored.
    pub prefix_dir: String,
    /// Timestamp, when the last chunk was received. Given as a unix epoch timestamp.
    pub last_chunk_received: i64
}

impl Transfer {
    /// Return a new `Transfer`
    ///
    /// # Arguments
    /// * `prefix`: Path where transferred chunks and assembled package will be stored.
    /// * `package`: [`PackageId`](../message/struct.PackageId.html) of this transfer.
    /// * `checksum`: SHA1 checksum of the fully assembled package.
    pub fn new(prefix: String, id: UpdateId, checksum: String)
        -> Transfer {
        Transfer {
            update_id: id,
            checksum: checksum,
            transferred_chunks: Vec::new(),
            prefix_dir: prefix,
            last_chunk_received: time::get_time().sec
        }
    }

    /// Create a transfer with empty values. To be used in tests.
    ///
    /// # Arguments
    /// * `prefix`: Path where transferred chunks and assembled package will be stored. This should
    ///   be a temporary directory for tests.
    #[cfg(test)]
    pub fn new_test(prefix: &PathPrefix) -> Transfer {
        Transfer {
            update_id: UpdateId::new(),
            checksum: "".to_string(),
            transferred_chunks: Vec::new(),
            prefix_dir: prefix.to_string(),
            last_chunk_received: time::get_time().sec
        }
    }

    /// Randomize a existing transfer, by creating a random
    /// [`PackageId`](../message/struct.PackageId.html). Returns the created `PackageId`, so it can
    /// be used in assertions.
    ///
    /// # Arguments
    /// * `i`: Size of the name and version strings.
    #[cfg(test)]
    pub fn randomize(&mut self, i: usize) -> UpdateId {
        let update_id = rand::thread_rng()
            .gen_ascii_chars().take(i).collect::<String>();

        trace!("Testing with:");
        trace!("  update_id: {}", update_id);
        self.update_id = update_id.clone();
        update_id
    }

    /// Write a transferred chunk to disk. Returns false and logs an error if something goes wrong.
    ///
    /// # Arguments
    /// * `msg`: Base64 encoded data of this chunk.
    /// * `index`: Index of this chunk
    pub fn write_chunk(&mut self,
                       msg: &str,
                       index: u64) -> bool {
        let success = msg.from_base64().map_err(|e| {
            error!("Could not decode chunk {} for update_id {}", index, self.update_id);
            error!("{}", e)
        }).and_then(|msg| self.get_chunk_path(index).map_err(|e| {
            error!("Could not get path for chunk {}", index);
            error!("{}", e)
        }).map(|path| {
            trace!("Saving chunk to {}", path.display());
            if write_new_file(&path, &msg) {
                self.transferred_chunks.push(index);
                self.transferred_chunks.sort();
                self.transferred_chunks.dedup();
                true
            } else {
                error!("Couldn't write chunk {} for update_id {}", index, self.update_id);
                false
            }
        })).unwrap_or(false);

        self.last_chunk_received = time::get_time().sec;
        success
    }

    /// Assemble the transferred chunks to a package and verify it with the provided checksum.
    /// Returns `false` and prints a error message if either the package can't be assembled or the
    /// checksum doesn't match.
    pub fn assemble_package(&self) -> Result<PathBuf, String> {
        trace!("Finalizing package {}", self.update_id);
        self.assemble_chunks().
            and_then(|_| {
                if self.checksum() {
                    self.get_package_path()
                } else {
                    Err(format!("Cannot assemble_package for update_id: {}", self.update_id))
                }
            })
    }

    /// Collect all chunks and concatenate them into one file. Returns a `String` with a error
    /// message, should something go wrong.
    fn assemble_chunks(&self) -> Result<(), String> {
        let package_path = try!(self.get_package_path());

        trace!("Saving update_id {} to {}", self.update_id, package_path.display());

        let mut file = try!(OpenOptions::new()
                               .write(true).append(true)
                               .create(true).truncate(true)
                               .open(package_path)
                               .map_err(|x| format!("Couldn't open file: {}", x)));

        let path: PathBuf = try!(self.get_chunk_dir());

        // Make sure all indices are valid and sort them
        let mut indices = Vec::new();
        for entry in try!(read_dir(&path)) {
            let entry = try!(entry.map_err(|x| format!("No entries: {}", x)));
            indices.push(try!(parse_index(entry)));
        }
        indices.sort();

        // Append indices to the final file
        for index in indices {
            try!(self.copy_chunk(&path, index, &mut file));
        }
        Ok(())
    }

    /// Read a chunk file file and append it to a package file. Returns a `String` with a error
    /// message should something go wrong.
    ///
    /// # Arguments
    /// * `path`: Pointer to a [`PathBuf`]
    ///   (https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html) where the chunks are
    ///   cached.
    /// * `index`: Index of the chunk to append.
    /// * `file`: Pointer to a `File` where the chunk should be appended. Should be created with
    ///   `OpenOptions` and the append only option. See the documentation for [`OpenOptions`]
    ///   (https://doc.rust-lang.org/stable/std/fs/struct.OpenOptions.html), [`File`]
    ///   (https://doc.rust-lang.org/stable/std/fs/struct.File.html), and the implementation of
    ///   [`assemble_chunks`](#method.assemble_chunks) for details.
    fn copy_chunk(&self, path: &PathBuf, index: u64, file: &mut File)
        -> Result<(), String> {
        let name = index.to_string();
        let mut chunk_path = path.clone();
        chunk_path.push(&name);
        let mut chunk =
            try!(OpenOptions::new().open(chunk_path)
                 .map_err(|x| format!("Couldn't open file: {}", x)));

        let mut buf = Vec::new();
        try!(chunk.read_to_end(&mut buf)
             .map_err(|x| format!("Couldn't read file {}: {}", name, x)));
        try!(file.write(&mut buf)
             .map_err(|x| format!("Couldn't write chunk {} to file {}: {}",
                                  name, self.update_id, x)));

        trace!("Wrote chunk {} to update_id {}", name, self.update_id);
        Ok(())
    }

    /// Verify the checksum of this transfer. Assumes the package was already assembled. Prints a
    /// error message showing the mismatched checksums and returns false on errors.
    fn checksum(&self) -> bool {
        let path = try_or!(self.get_package_path(), return false);
        let mut file = try_or!(OpenOptions::new().open(path), return false);
        let mut data = Vec::new();

        // TODO: avoid reading in the whole file at once
        try_msg_or!(file.read_to_end(&mut data),
                    "Couldn't read file to check",
                    return false);

        let mut hasher = Sha1::new();
        hasher.input(&data);
        let hash = hasher.result_str();

        if hash == self.checksum {
            true
        } else {
            error!("Checksums didn't match for update_id {}", self.update_id);
            error!("    Expected: {}", self.checksum);
            error!("    Got: {}", hash);
            false
        }
    }

    /// Get the full path for the specified chunk index. Returns a
    /// [`PathBuf`](https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html) on success or a
    /// `String` on errors detailing what went wrong.
    ///
    /// # Arguments
    /// * `index`: The index for which the path should be constructed
    fn get_chunk_path(&self, index: u64) -> Result<PathBuf, String> {
        let mut path = try!(self.get_chunk_dir());
        let filename = index.to_string();

        trace!("Using filename {}", filename);
        path.push(filename);
        Ok(path)
    }

    /// Get the full path for the package of this `Transfer`. Returns a
    /// [`PathBuf`](https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html) on success or a
    /// `String` on errors detailing what went wrong.
    fn get_package_path(&self) -> Result<PathBuf, String> {
        let mut path = try!(self.get_package_dir());
        path.push(format!("{}.spkg", self.update_id));
        Ok(path)
    }

    /// Get the directory, where this `Transfer` caches chunks. Returns a
    /// [`PathBuf`](https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html) on success or a
    /// `String` on errors detailing what went wrong.
    fn get_chunk_dir(&self) -> Result<PathBuf, String> {
        let mut path = PathBuf::from(&self.prefix_dir);
        path.push("downloads");
        path.push(format!("{}", self.update_id));

        fs::create_dir_all(&path).map_err(|e| {
            let path_str = path.to_str().unwrap_or("unknown");
            format!("Couldn't create chunk dir at '{}': {}", path_str, e)
        }).map(|_| path)
    }

    /// Get the directory, where this `Transfer` stores the assembled package. Returns a
    /// [`PathBuf`](https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html) on success or a
    /// `String` on errors detailing what went wrong.
    fn get_package_dir(&self) -> Result<PathBuf, String> {
        let mut path = PathBuf::from(&self.prefix_dir);
        path.push("packages");

        fs::create_dir_all(&path).map_err(|e| {
            let path_str = path.to_str().unwrap_or("unknown");
            format!("Couldn't create packges dir at '{}': {}", path_str, e)
        }).map(|_| path)
    }
}

impl Drop for Transfer {
    /// When a `Transfer` is freed it will also clear out the associated chunk cache on disk.
    fn drop(&mut self) {
        let dir = try_or!(self.get_chunk_dir(), return);
        trace!("Dropping transfer for package {}", self.update_id);

        for entry in try_or!(read_dir(&dir), return) {
            let entry = try_or!(entry, continue);
            let _ = entry.file_name().into_string().map_err(|_|
                error!("Found a malformed entry!")
            ).map(|name| {
                trace!("Dropping chunk file {}", name);
                try_or!(fs::remove_file(entry.path()), return);
            });
        }

        try_or!(fs::remove_dir(dir), return);
    }
}

/// Write the provided `data` to the file at `path`. Will create the file if it doesn't exist and
/// overwrite existing files. Returns `false` on errors, after logging a error message.
///
/// # Arguments
/// * `path`: Pointer to a [`PathBuf`]
///   (https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html) where the data will be
///   written to. Needs to point to a (possibly nonexistent) file.
/// * `data`: The data to be written to disk.
fn write_new_file(path: &PathBuf, data: &Vec<u8>) -> bool {
    let mut file = try_or!(OpenOptions::new()
                           .write(true).create(true)
                           .truncate(true).open(path),
                           return false);

    try_or!(file.write_all(data), return false);
    try_or!(file.flush(), return false);
    true
}

/// Read the contents of a directory. Returns a
/// [`ReadDir`](https://doc.rust-lang.org/stable/std/fs/struct.ReadDir.html) iterator on success or
/// a `String` with a detailed error message on failure.
fn read_dir(path: &PathBuf) -> Result<fs::ReadDir, String> {
    fs::read_dir(path).map_err(|e| {
        let path_str = path.to_str().unwrap_or("unknown");
        format!("Couldn't read dir at '{}': {}", path_str, e)
    })
}

/// Parse a [`DirEntry`](https://doc.rust-lang.org/stable/std/fs/struct.DirEntry.html) to a `u64`.
/// Returns the parsed number on success or a `String` with a detailed error message on failure.
///
/// # Arguments
/// * `entry`: `DirEntry` to be parsed.
fn parse_index(entry: DirEntry) -> Result<u64, String> {
    let name = entry.file_name().into_string()
        .unwrap_or("unknown".to_string());
    u64::from_str(&name)
        .map_err(|_| "Couldn't parse chunk index from filename".to_string())
}

use std::collections::HashMap;

/// Type alias to hide the internal `HashMap`, that is used to store
/// [`Transfer`](../persistence/struct.Transfer.html)s.
pub struct Transfers {
    items: HashMap<UpdateId, Transfer>,
    storage_dir: String
}

impl Transfers {
    pub fn new(dir: String) -> Transfers {
        Transfers {
            items: HashMap::new(),
            storage_dir: dir
        }
    }

    pub fn get(&self, pkg: &UpdateId) -> Option<&Transfer> {
        self.items.get(pkg)
    }

    pub fn get_mut(&mut self, pkg: &UpdateId) -> Option<&mut Transfer> {
        self.items.get_mut(pkg)
    }

    pub fn push(&mut self, pkg: UpdateId, cksum: String) {
        self.items.insert(
            pkg.clone(),
            Transfer::new(self.storage_dir.to_string(), pkg, cksum));
    }

    #[cfg(test)]
    pub fn push_test(&mut self, tr: Transfer) {
        self.items.insert(tr.update_id.clone(), tr);
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn remove(&mut self, pkg: &UpdateId) {
        self.items.remove(pkg);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn prune(&mut self, now: i64, timeout: i64) {
        self.items.iter()
            .filter(|&(_, v)| now - v.last_chunk_received > timeout)
            .map(|(k, _)| k.clone())
            .collect::<Vec<UpdateId>>()
            .iter().map(|k| {
                self.items.remove(k);
                info!("Transfer for update_id {} timed out after {} ms", k, timeout)})
            .collect::<Vec<()>>();
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use test_library::*;

    use std::path::PathBuf;
    use std::fs;
    use std::fs::OpenOptions;
    use std::io::prelude::*;

    use rand;
    use rand::Rng;
    use rustc_serialize::base64;
    use rustc_serialize::base64::ToBase64;

    fn create_tmp_directories(prefix: &PathPrefix) {
        for i in 1..20 {
            let mut transfer = Transfer::new_test(prefix);
            let update_id = transfer.randomize(i);
            let chunk_dir: PathBuf = transfer.get_chunk_dir().unwrap();
            let path = format!("{}/downloads/{}", prefix, update_id);
            assert_eq!(chunk_dir.to_str().unwrap(), path);

            let path = PathBuf::from(path);
            // This also makes sure it's a directory
            let dir = fs::read_dir(&path).unwrap();

            for _ in dir {
                panic!("Found non-empty directory!");
            }
        }
    }

    #[test]
    fn it_creates_a_tmp_directory() {
        test_init!();
        let prefix = PathPrefix::new();
        create_tmp_directories(&prefix);
    }

    #[test]
    fn it_cleans_up_the_tmp_directories() {
        test_init!();
        let prefix = PathPrefix::new();
        create_tmp_directories(&prefix);
        let path = PathBuf::from(format!("{}/downloads/", prefix));
        let dir = fs::read_dir(&path).unwrap();

        for _ in dir {
            panic!("Found non-empty directory!");
        }
    }

    #[test]
    fn it_creates_a_persistent_directory_per_package() {
        test_init!();
        let prefix = PathPrefix::new();
        for i in 1..20 {
            let mut transfer = Transfer::new_test(&prefix);
            let update_id = transfer.randomize(i);

            let chunk_dir: PathBuf = transfer.get_package_path().unwrap();
            let path = format!("{}/packages/{}.spkg", prefix, update_id);
            assert_eq!(chunk_dir.to_str().unwrap(), path);
        }
    }

    macro_rules! assert_chunk_written {
        ($transfer:ident,
         $prefix:ident,
         $update_id:ident,
         $index:ident,
         $data:ident) => {{
            trace!("Testing with: {}", $data);

            let b64_data = $data.as_bytes().to_base64(
                base64::Config {
                    char_set: base64::CharacterSet::UrlSafe,
                    newline: base64::Newline::LF,
                    pad: true,
                    line_length: None
                });

            trace!("Encoded as: {}", b64_data);

            $transfer.write_chunk(&b64_data, $index as u64);

            let path = format!("{}/downloads/{}/{}", $prefix, $update_id, $index);

            trace!("Expecting file at: {}", path);

            let mut from_disk = Vec::new();
            OpenOptions::new()
                .open(PathBuf::from(path))
                .unwrap()
                .read_to_end(&mut from_disk)
                .unwrap();

            assert_eq!($data.into_bytes(), from_disk);
        }}
    }

    #[test]
    fn it_writes_decoded_data_to_disk() {
        test_init!();
        let prefix = PathPrefix::new();
        for i in 1..20 {
            let mut transfer = Transfer::new_test(&prefix);
            let update_id = transfer.randomize(i);
            for i in 1..20 {
                let data = rand::thread_rng()
                    .gen_ascii_chars().take(i).collect::<String>();
                assert_chunk_written!(transfer, prefix, update_id, i, data);
            }
        }
    }

    #[test]
    fn it_correctly_assembles_stored_chunks() {
        test_init!();
        let prefix = PathPrefix::new();
        for i in 1..20 {
            let mut transfer = Transfer::new_test(&prefix);
            let update_id = transfer.randomize(i);
            let mut full_data = String::new();
            for i in 1..20 {
                let data = rand::thread_rng()
                    .gen_ascii_chars().take(i).collect::<String>();
                full_data.push_str(&data);

                assert_chunk_written!(transfer, prefix, update_id, i, data);
            }

            transfer.assemble_chunks().unwrap();

            let path = format!("{}/packages/{}.spkg", prefix, update_id);

            trace!("Expecting assembled file at: {}", path);

            let mut from_disk = Vec::new();
            OpenOptions::new()
                .open(PathBuf::from(path))
                .unwrap()
                .read_to_end(&mut from_disk)
                .unwrap();

            assert_eq!(full_data.into_bytes(), from_disk);
        }
    }

    fn checksum_matching(data: String, checksum: String) -> bool {
            let prefix = PathPrefix::new();
            let mut transfer = Transfer::new_test(&prefix);
            let update_id = transfer.randomize(20);
            let index = 0;
            assert_chunk_written!(transfer, prefix, update_id, index, data);
            transfer.assemble_chunks().unwrap();

            transfer.checksum = checksum;
            transfer.checksum()
    }

    #[test]
    fn it_returns_true_for_correct_checksums() {
        test_init!();
        assert!(checksum_matching("test\n".to_string(),
        "4e1243bd22c66e76c2ba9eddc1f91394e57f9f83".to_string()));
    }

    #[test]
    fn it_returns_false_for_incorrect_checksums() {
        test_init!();
        assert!(!checksum_matching("test\n".to_string(),
        "fa7c4d75bae3a641d1f9ab5df028175bfb8a69ca".to_string()));
    }

    #[test]
    fn it_returns_false_for_invalid_checksums() {
        test_init!();
        assert!(!checksum_matching("test\n".to_string(),
        "invalid".to_string()));
    }
}
