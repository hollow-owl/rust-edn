mod edn_compare;
mod edn_pest;
mod edn_reader;

use std::{
    fs,
    io::{self, Read, Write},
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
        let r_edn = rust_edn(&input);
        dbg!(&r_edn);
        let c_edn = clojure_edn(&input);
        dbg!(&c_edn);
        if r_edn != c_edn {
            println!("ERROR: rust and clojure implementations do not match");
        }
    }
}

// {:country "GB", :gid #uuid "7d0162a9-2636-46f2-b0e8-a9336075eee2", :sort_name "His Master's Voice", :name "His Master's Voice", :type "Original Production"}
fn main() {
    repl();
    // let failing_files = vec![
    //     "examples/output/chat.edn",
    //     "examples/output/config.edn.2",
    //     "examples/output/create-access-token.edn",
    //     "examples/output/de.edn",
    //     "examples/output/de.edn.1",
    //     "examples/output/de.edn.2",
    //     "examples/output/default-rc.edn.1",
    //     "examples/output/dellstore-schema.edn",
    //     "examples/output/demo3.001.edn",
    //     "examples/output/double-ls.edn",
    //     "examples/output/double-ls.edn.1",
    //     "examples/output/double-ls.edn.2",
    //     "examples/output/double-ls.edn.3",
    //     "examples/output/double-ls.edn.4",
    //     "examples/output/example_list.edn",
    //     "examples/output/imperfect.edn",
    //     "examples/output/ja.edn",
    //     "examples/output/maps_unrecognized_keys.edn",
    //     "examples/output/number.edn",
    //     "examples/output/number.edn.1",
    //     "examples/output/number.edn.2",
    //     "examples/output/pipeline-with-includes.edn",
    //     "examples/output/pll_64M.edn",
    //     "examples/output/production.edn",
    //     "examples/output/provenance.edn",
    //     "examples/output/provenance.edn.1",
    //     "examples/output/provenance.edn.2",
    //     "examples/output/put-resource.edn",
    //     "examples/output/questions-2019.edn",
    //     "examples/output/RAM.edn",
    //     "examples/output/sample_data.edn",
    //     "examples/output/sample.edn.1",
    //     "examples/output/schema.edn.1",
    //     "examples/output/seattle-data1.edn",
    //     "examples/output/seattle-data1.edn.1",
    //     "examples/output/seattle-data1.edn.2",
    //     "examples/output/shadow-cljs.edn.17",
    //     "examples/output/shadow-cljs.edn.22",
    //     "examples/output/spfloat-comp.edn",
    //     "examples/output/spfloat-comp.edn.1",
    //     "examples/output/spfloat-comp.edn.2",
    //     "examples/output/spfloat-comp.edn.3",
    //     "examples/output/spfloat-comp.edn.4",
    //     "examples/output/system.edn",
    //     "examples/output/temperature.edn",
    //     "examples/output/time.edn",
    //     "examples/output/traefik.edn",
    // ];
    // for f in failing_files.into_iter() {
    //     let file = fs::read_to_string(f).expect("could not read file");
    //     if !is_match(file.as_str()) {
    //         println!("Failed on {f}")
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_output() {
        let ignore = vec![
            "./examples/output/spfloat-comp.edn",
            "./examples/output/spfloat-comp.edn.1",
            "./examples/output/spfloat-comp.edn.2",
            "./examples/output/spfloat-comp.edn.3",
            "./examples/output/spfloat-comp.edn.4",
            // Untested
            "./examples/output/RAM.edn",
            "./examples/output/double-ls.edn.2",
            "./examples/output/double-ls.edn.4",
            "./examples/output/imperfect.edn",
            "./examples/output/.#shadow-cljs.edn.1",
            "./examples/output/pll_64M.edn",
        ]
        .into_iter()
        .collect::<HashSet<&str>>();

        // ./examples/output/double-ls.edn.1 does not match
        let paths = fs::read_dir("./examples/output").unwrap();
        for path in paths {
            let path = path.unwrap().path();
            let path_str = path.as_os_str().to_str().unwrap();
            if ignore.contains(&path_str) {
                println!("Ignoring {path_str}");
                continue;
            }
            println!("Testing {path_str}");
            let contents = fs::read_to_string(&path).unwrap();
            let r_edn = rust_edn(&contents);
            let c_edn = clojure_edn(&contents);
            let cr_edn = c_edn.and_then(|x| rust_edn(&x));
            if r_edn != cr_edn {
                println!("{} does not match", path_str);
            }
            assert_eq!(r_edn, cr_edn);
        }
    }
}
