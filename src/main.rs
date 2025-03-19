mod edn_compare;
mod edn_pest;
mod edn_reader;

use std::{
    fs,
    io::{self, Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use edn_compare::{clojure_edn, rust_edn};

fn repl() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        if input.starts_with("file ") {
            let path = input.trim_start_matches("file ").trim();
            dbg!(path);
            let r_edn = edn_file(path).unwrap();
            println!("{r_edn}");
            continue;
        }
        let r_edn = rust_edn(&input);
        dbg!(&r_edn);
        let c_edn = clojure_edn(&input);
        dbg!(&c_edn);
        // if r_edn != c_edn {
        //     println!("ERROR: rust and clojure implementations do not match");
        // }
    }
}

fn edn_file<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let contents = fs::read_to_string(&path).unwrap();
    rust_edn(&contents)
}

fn main() {
    repl();
    // let path = "./test.edn";
    // let contents = fs::read_to_string(&path).unwrap();
    // let r_edn = rust_edn(&contents).unwrap();
    // println!("{r_edn}");
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use fs::File;
    use text_diff::{assert_diff, diff};

    use super::*;

    // let ignore = vec![
    //     "./examples/output/spfloat-comp.edn",
    //     "./examples/output/spfloat-comp.edn.1",
    //     "./examples/output/spfloat-comp.edn.2",
    //     "./examples/output/spfloat-comp.edn.3",
    //     "./examples/output/spfloat-comp.edn.4",
    //     // Untested
    //     "./examples/output/RAM.edn",
    //     "./examples/output/double-ls.edn.2",
    //     "./examples/output/double-ls.edn.4",
    //     "./examples/output/imperfect.edn",
    //     "./examples/output/.#shadow-cljs.edn.1",
    //     "./examples/output/pll_64M.edn",
    //     // Panic No reader function
    //     "./examples/output/shadow-cljs.edn.17",
    //     "./examples/output/demo3.001.edn",
    //     "./examples/output/create-access-token.edn",
    //     "./examples/output/traefik.edn",
    //     "./examples/output/questions-2019.edn",
    //     // Not Matching
    //     // Clojure converts regular map to namespace map when keys have namespace
    //     "./examples/output/process.edn.1",
    //     "./examples/output/assignment.edn.4",
    //     "./examples/output/deps.edn.20",
    //     "./examples/output/config.edn.6",
    //     // Not Matching - namespace maps in edn
    //     "./examples/output/tmp.edn",
    //     "./examples/output/descriptor.edn.31",
    //     "./examples/output/descriptor.edn.20",
    //     "./examples/output/descriptor.edn.14",
    //     "./examples/output/descriptor.edn.28",
    //     "./examples/output/tmp.edn",
    //     "./examples/output/descriptor.edn.31",
    //     "./examples/output/dmu.edn",
    //     "./examples/output/descriptor.edn.11",
    //     "./examples/output/c2d_ggprism.edn",
    //     "./examples/output/descriptor.edn.19",
    //     "./examples/output/deps.edn.10",
    //     "./examples/output/descriptor.edn.17",
    //     "./examples/output/descriptor.edn.3",
    //     "./examples/output/descriptor.edn.27",
    //     "./examples/output/ddm.edn",
    //     "./examples/output/descriptor.edn.22",
    //     "./examples/output/descriptor.edn.12",
    //     "./examples/output/descriptor.edn.13",
    //     "./examples/output/descriptor.edn.26",
    //     "./examples/output/c2d_imagej.edn",
    //     "./examples/output/descriptor.edn.15",
    //     "./examples/output/descriptor.edn.2",
    //     "./examples/output/descriptor.edn.23",
    //     "./examples/output/descriptor.edn.1",
    //     "./examples/output/gs1.edn",
    //     "./examples/output/descriptor.edn.29",
    //     "./examples/output/descriptor.edn.7",
    //     "./examples/output/descriptor.edn.5",
    //     "./examples/output/descriptor.edn.9",
    //     "./examples/output/descriptor.edn.30",
    //     "./examples/output/descriptor.edn.4",
    //     "./examples/output/descriptor.edn.10",
    //     "./examples/output/exo.edn",
    //     "./examples/output/cm2.edn",
    //     "./examples/output/descriptor.edn.24",
    //     "./examples/output/descriptor.edn.8",
    //     "./examples/output/descriptor.edn.6",
    //     "./examples/output/plc.edn",
    //     "./examples/output/wc99.edn",
    //     "./examples/output/c2d_pj_3.edn",
    //     "./examples/output/descriptor.edn.21",
    //     "./examples/output/descriptor.edn.16",
    //     "./examples/output/descriptor.edn.18",
    //     "./examples/output/descriptor.edn.25",
    //     "./examples/output/descriptor.edn",
    // ]
    // .into_iter()
    // .collect::<HashSet<&str>>();

    #[test]
    fn test_valid_edn_files() {
        let mut valid_file = File::create("valid_edn.txt").unwrap();
        let mut invalid_file = File::create("invalid_edn.txt").unwrap();

        let paths = fs::read_dir("./examples/output").unwrap();
        for path in paths {
            let path = path.unwrap().path();
            let path_str = path.as_os_str().to_str().unwrap();
            println!("Testing {path_str}");
            let contents = fs::read_to_string(&path).unwrap();
            let c_edn = clojure_edn(&contents);
            if c_edn.is_ok() {
                writeln!(valid_file, "{path_str}").unwrap();
            } else {
                writeln!(invalid_file, "{path_str}").unwrap();
            }
        }
    }

    #[test]
    fn test_invalid_edn_reasons() {
        let mut invalid_no_tags = File::create("invlaid_no_tags_edn.txt").unwrap();
        let paths = fs::read_to_string("invalid_edn.txt").unwrap();

        for path in paths.lines() {
            let contents = fs::read_to_string(&path).unwrap();
            let c_edn = clojure_edn(&contents);
            if let Err(err_msg) = c_edn {
                if err_msg.contains("No reader function for tag") {
                    continue;
                } else {
                    writeln!(invalid_no_tags, "{path} | {err_msg}").unwrap();
                }
            } else {
                println!("{path} not error!")
            }
        }
    }

    #[test]
    fn test_invalid_no_tags_output() {
        let paths = fs::read_to_string("invalid_no_tags.txt").unwrap();
        for line in paths.lines() {
            let (path, reason) = line.split_once(" | ").unwrap();
            let contents = fs::read_to_string(&path).unwrap();
            let r_edn = rust_edn(&contents);
        }
    }
    #[test]
    fn test_valid_output() {
        let ignore = vec!["./examples/output/tag-inst.edn"]
            .into_iter()
            .collect::<HashSet<&str>>();
        let paths = fs::read_to_string("valid_edn.txt").unwrap();
        for path in paths.lines() {
            println!("Testing {path}");
            if ignore.contains(path) {
                println!("Skipping {path}");
                continue;
            }
            let contents = fs::read_to_string(&path).unwrap();
            let r_edn = rust_edn(&contents);
            let c_edn = clojure_edn(&contents);
            let cr_edn = c_edn.map(|x| rust_edn(&x)).unwrap();

            // rust parse -> rust serialize == clojure parse -> clojure serialize
            // rust parse -> rust serialize -> clojure parse -> clojure serialize == clojure parse -> clojure serialize -> rust parse -> rust serializeo

            // rust parse -> rust serialize == clojure parse -> clojure serialize -> rust parse -> rust serialize
            if let (Ok(r), Ok(c)) = (r_edn.as_deref(), cr_edn.as_deref()) {
                let (diff, change) = diff(r, c, " ");
                if diff != 0 {
                    println!("{path} does not match");
                }
            } else {
                dbg!((&r_edn, &cr_edn));
                panic!("{path} Not success");
            }
        }
    }
}
