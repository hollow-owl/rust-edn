mod edn_compare;
mod edn_pest;
mod edn_reader;

use std::{
    env, fs,
    io::{self, Write},
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
            let contents = fs::read_to_string(&path).unwrap();
            let r_edn = rust_edn(&contents).unwrap();
            println!("{r_edn}");
            continue;
        }
        let r_edn = rust_edn(&input);
        dbg!(&r_edn);
        let c_edn = clojure_edn(&input);
        dbg!(&c_edn);
    }
}

fn main() {
    let args = env::args().into_iter().collect::<Vec<String>>();
    let path = {
        let mut file_path = None;
        for i in 0..args.len() {
            if args[i] == "--file" {
                file_path = args.get(i + 1);
            }
        }
        file_path
    };
    if let Some(path) = path {
        let contents = fs::read_to_string(&path).unwrap();
        let r_edn = rust_edn(&contents).unwrap();
        println!("{r_edn}");
    } else {
        repl();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use fs::File;
    use text_diff::{diff, print_diff};

    use super::*;

    #[ignore = "Only used to categorize files in ./examples/output"]
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
            match c_edn {
                Ok(_) => writeln!(valid_file, "{path_str}").unwrap(),
                Err(err_msg) => writeln!(invalid_file, "{path_str} | {err_msg}").unwrap(),
            }
        }
    }

    #[test]
    fn test_invalid_output() {
        let paths = fs::read_to_string("invalid_edn.txt").unwrap();

        for line in paths.lines() {
            let (path, reason) = line.split_once(" | ").unwrap();
            println!("Testing {path}");
            let contents = fs::read_to_string(&path).unwrap();
            let r_edn = rust_edn(&contents);

            match r_edn.as_deref() {
                Ok(out) => assert_eq!(out, reason),
                Err(out) => assert_eq!(out, reason),
            }
        }
    }
    #[test]
    fn test_valid_output() {
        let mut success = true;
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
                let (diff, _) = diff(r, c, " ");
                if diff != 0 {
                    print_diff(r, c, " ");
                    println!("{path} does not match");
                    success = false
                }
            } else {
                dbg!((&r_edn, &cr_edn));
                panic!("{path} Not success");
            }
        }
        assert!(success);
    }
}
