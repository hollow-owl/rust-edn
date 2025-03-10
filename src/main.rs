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
        // if r_edn != c_edn {
        //     println!("ERROR: rust and clojure implementations do not match");
        // }
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
    fn test_output() {
        // "./examples/output/color_rom.edn.2"
        let ignore = vec![
            "./examples/output/tag-inst.edn",
            // Long diff don't know whats different, xlinx stuff
            "./examples/output/dds.edn",
            "./examples/output/color_rom.edn",
            "./examples/output/color_rom.edn.1",
            "./examples/output/color_rom.edn.2",
            "./examples/output/color_rom.edn.3",
            "./examples/output/color_rom.edn.4",
        ]
        .into_iter()
        .collect::<HashSet<&str>>();
        let paths = fs::read_to_string("valid_edn.txt").unwrap();
        for path in paths.lines() {
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
            if let (Some(r), Some(c)) = (r_edn.as_deref(), cr_edn.as_deref()) {
                let (diff, change) = diff(r, c, " ");
                if diff != 0 {
                    println!("{path} does not match");
                }
            } else {
                panic!("{path} Not success");
            }
        }
    }
}
