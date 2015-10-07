use std::fs;
use std::fs::OpenOptions;
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

use message::PackageId;

// TODO: refactor into submodules

pub struct Transfer {
    pub package: PackageId,
    pub checksum: String,
    pub transferred_chunks: Vec<u64>,
    pub prefix_dir: String,
    pub last_chunk_received: i64
}

impl Transfer {
    pub fn new(prefix: String, package: PackageId, checksum: String)
        -> Transfer {
        Transfer {
            package: package,
            checksum: checksum,
            transferred_chunks: Vec::new(),
            prefix_dir: prefix,
            last_chunk_received: time::get_time().sec
        }
    }

    #[cfg(test)]
    pub fn new_test(prefix: &PathPrefix) -> Transfer {
        Transfer {
            package: PackageId {
                name: "".to_string(),
                version: "".to_string()
            },
            checksum: "".to_string(),
            transferred_chunks: Vec::new(),
            prefix_dir: prefix.to_string(),
            last_chunk_received: time::get_time().sec
        }
    }

    #[cfg(test)]
    pub fn randomize(&mut self, i: usize) -> PackageId {
        let name = rand::thread_rng()
            .gen_ascii_chars().take(i).collect::<String>();
        let version = rand::thread_rng()
            .gen_ascii_chars().take(i).collect::<String>();

        trace!("Testing with:");
        trace!("  name: {}\n  version {}", name, version);

        self.package.name = name.clone();
        self.package.version = version.clone();

        PackageId {
            name: name,
            version: version
        }
    }

    pub fn assemble_package(&self) -> bool {
        trace!("Finalizing package {}", self.package);
        try_or!(self.assemble_chunks(), return false);
        self.checksum()
    }

    pub fn write_chunk(&mut self,
                       msg: &str,
                       index: u64) -> bool {
        let success = msg.from_base64().map_err(|e| {
            error!("Could not decode chunk {} for package {}", index, self.package);
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
                error!("Couldn't write chunk {} for package {}", index, self.package);
                false
            }
        })).unwrap_or(false);

        self.last_chunk_received = time::get_time().sec;
        success
    }

    fn get_chunk_path(&self, index: u64) -> Result<PathBuf, String> {
        let mut path = try!(self.get_chunk_dir());
        let filename = index.to_string();

        trace!("Using filename {}", filename);
        path.push(filename);
        Ok(path)
    }

    fn get_package_path(&self) -> Result<PathBuf, String> {
        let mut path = try!(self.get_package_dir());
        path.push(format!("{}.spkg", self.package));
        Ok(path)
    }

    fn assemble_chunks(&self) -> Result<(), String> {
        let package_path = try!(self.get_package_path());

        trace!("Saving package {} to {}", self.package, package_path.display());

        let mut file = try!(OpenOptions::new()
                               .write(true).append(true)
                               .create(true).truncate(true)
                               .open(package_path)
                               .map_err(|x| format!("Couldn't open file: {}", x)));

        let path: PathBuf = try!(self.get_chunk_dir());

        let mut indices = Vec::new();
        for entry in try!(read_dir(&path)) {
            let entry = try!(entry.map_err(|x| format!("No entries: {}", x)));
            let name  = entry.file_name().into_string()
                .unwrap_or("unknown".to_string());

            let chunk_index =
                try!(u64::from_str(&name)
                     .map_err(|_| "Couldn't parse chunk index from filename"
                              .to_string()));
            indices.push(chunk_index);
        }
        indices.sort();

        for index in indices {
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
                                      name, self.package, x)));

            trace!("Wrote chunk {} to package {}", name, self.package);
        }
        Ok(())
    }

    fn get_chunk_dir(&self) -> Result<PathBuf, String> {
        let mut path = PathBuf::from(&self.prefix_dir);
        path.push("downloads");
        path.push(format!("{}", self.package));

        fs::create_dir_all(&path).map_err(|e| {
            let path_str = path.to_str().unwrap_or("unknown");
            format!("Couldn't create chunk dir at '{}': {}", path_str, e)
        }).map(|_| path)
    }

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
            error!("Checksums didn't match for package {}", self.package);
            error!("    Expected: {}", self.checksum);
            error!("    Got: {}", hash);
            false
        }
    }

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
    fn drop(&mut self) {
        let dir = try_or!(self.get_chunk_dir(), return);
        trace!("Dropping transfer for package {}", self.package);

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

fn write_new_file(path: &PathBuf, data: &Vec<u8>) -> bool {
    let mut file = try_or!(OpenOptions::new()
                           .write(true).create(true)
                           .truncate(true).open(path),
                           return false);

    try_or!(file.write_all(data), return false);
    try_or!(file.flush(), return false);
    true
}

fn read_dir(path: &PathBuf) -> Result<fs::ReadDir, String> {
    fs::read_dir(path).map_err(|e| {
        let path_str = path.to_str().unwrap_or("unknown");
        format!("Couldn't read dir at '{}': {}", path_str, e)
    })
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
            let package = transfer.randomize(i);
            let chunk_dir: PathBuf = transfer.get_chunk_dir().unwrap();
            let path = format!("{}/downloads/{}-{}", prefix,
                               package.name, package.version);
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
            let package = transfer.randomize(i);

            let chunk_dir: PathBuf = transfer.get_package_path().unwrap();
            let path = format!("{}/packages/{}-{}.spkg", prefix,
                               package.name, package.version);
            assert_eq!(chunk_dir.to_str().unwrap(), path);
        }
    }

    macro_rules! assert_chunk_written {
        ($transfer:ident,
         $prefix:ident,
         $package:ident,
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

            let path = format!("{}/downloads/{}-{}/{}", $prefix,
                                $package.name, $package.version, $index);

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
            let package = transfer.randomize(i);
            for i in 1..20 {
                let data = rand::thread_rng()
                    .gen_ascii_chars().take(i).collect::<String>();
                assert_chunk_written!(transfer, prefix, package, i, data);
            }
        }
    }

    #[test]
    fn it_correctly_assembles_stored_chunks() {
        test_init!();
        let prefix = PathPrefix::new();
        for i in 1..20 {
            let mut transfer = Transfer::new_test(&prefix);
            let package = transfer.randomize(i);
            let mut full_data = String::new();
            for i in 1..20 {
                let data = rand::thread_rng()
                    .gen_ascii_chars().take(i).collect::<String>();
                full_data.push_str(&data);

                assert_chunk_written!(transfer, prefix, package, i, data);
            }

            transfer.assemble_chunks().unwrap();

            let path = format!("{}/packages/{}-{}.spkg", prefix,
                               package.name, package.version);

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
            let package = transfer.randomize(20);
            let index = 0;
            assert_chunk_written!(transfer, prefix, package, index, data);
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
