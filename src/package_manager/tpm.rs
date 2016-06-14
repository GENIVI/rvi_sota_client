use std::fmt::Debug;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::prelude::*;
use std::sync::mpsc::Receiver;

use datatype::{Error, Package, UpdateResultCode};
use package_manager::package_manager::InstallOutcome;


pub fn installed_packages(path: &str) -> Result<Vec<Package>, Error> {
    let f        = try!(File::open(path));
    let reader   = BufReader::new(f);
    let mut pkgs = Vec::new();

    for line in reader.lines() {
        let line  = try!(line);
        let parts = line.split(' ');

        if parts.clone().count() == 2 {
            if let Some(name) = parts.clone().nth(0) {
                if let Some(version) = parts.clone().nth(1) {
                    pkgs.push(Package {
                        name:    name.to_string(),
                        version: version.to_string()
                    });
                }
            }
        }
    }

    Ok(pkgs)
}

pub fn install_package(path: &str, pkg: &str, succeeds: bool) -> Result<InstallOutcome, InstallOutcome> {
    let install = || -> Result<(), Error> {
        let f = OpenOptions::new().create(true).write(true).append(true).open(path)
            .expect("couldn't open file for writing");
        let mut writer = BufWriter::new(f);
        try!(writer.write(pkg.as_bytes()));
        try!(writer.write(b"\n"));
        Ok(())
    };

    if succeeds {
        match install() {
            Ok(_)  => Ok((UpdateResultCode::OK, "".to_string())),
            Err(e) => Err((UpdateResultCode::INSTALL_FAILED, format!("{:?}", e)))
        }
    } else {
        Err((UpdateResultCode::INSTALL_FAILED, "failed".to_string()))
    }
}

pub fn assert_rx<X: PartialEq + Debug>(rx: Receiver<X>, xs: &[X]) {
    let n      = xs.len();
    let mut xs = xs.iter();
    for _ in 0..n {
        let val = rx.recv().expect("assert_rx expected another val");
        let x   = xs.next().expect(&format!("assert_rx: no match for val: {:?}", val));
        assert_eq!(val, *x);
    }
}


#[cfg(test)]
mod tests {

    use std::fs;
    use std::fs::File;
    use std::io::prelude::*;

    use super::*;
    use datatype::Package;

    fn pkg1() -> Package {
        Package {
            name:    "apa".to_string(),
            version: "0.0.0".to_string()
        }
    }

    fn pkg2() -> Package {
        Package {
            name:    "bepa".to_string(),
            version: "1.0.0".to_string()
        }
    }

    #[test]
    fn test_installed_packages() {

        let path = "/tmp/test1";

        let mut f = File::create(path).unwrap();

        f.write(b"apa 0.0.0\n").unwrap();
        f.write(b"bepa 1.0.0").unwrap();

        assert_eq!(installed_packages(path).unwrap(), vec!(pkg1(), pkg2()));

    }

    #[test]
    fn bad_installed_packages() {

        let path = "/tmp/test2";

        let mut f = File::create(path).unwrap();

        f.write(b"cepa-2.0.0\n").unwrap();

        assert_eq!(installed_packages(path).unwrap(), Vec::new());

    }

    #[test]
    fn test_install_package() {

        let path = "/tmp/test3";

        let _ = fs::remove_file(path);

        install_package(path, "apa 0.0.0", true).unwrap();
        install_package(path, "bepa 1.0.0", true).unwrap();

        assert_eq!(installed_packages(path).unwrap(), vec!(pkg1(), pkg2()));

    }

    #[test]
    fn test_install_package_fails() {

        let path = "/tmp/test4";

        let _ = fs::remove_file(path);

        let _ = install_package(path, "apa 0.0.0", false);
        install_package(path, "bepa 1.0.0", true).unwrap();

        assert_eq!(installed_packages(path).unwrap(), vec!(pkg2()));

    }
}
