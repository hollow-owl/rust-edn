mod edn_pest;
mod edn_reader;
use std::{
    fs,
    io::{self, Read, Write},
    process::{Command, Stdio},
};

fn repl() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        if is_match(input.trim()) {
            println!("Match");
        } else {
            println!("ERROR: {input} does not match")
        }
    }
}

fn test_file(path: &str) {
    let file = fs::read_to_string(path).expect("could not read file");
    if is_match(file.trim()) {
        println!("Match");
    } else {
        println!("ERROR: File {path} does not match")
    }
}

fn is_match(input: &str) -> bool {
    let input = input.trim().to_owned();
    let clojure_edn = clojure_edn(input.as_str());
    let edn = edn_reader::read_str(input);
    // dbg!((&edn, &clojure_edn));
    match (&edn, &clojure_edn) {
        (Some(_), Ok(_)) => true,
        (None, Err(clojure_out)) => {
            dbg!(&clojure_out);
            true
        }
        _ => {
            dbg!(edn, clojure_edn);
            false
        }
    }
}

fn clojure_edn(input: &str) -> Result<String, (String, String)> {
    let mut child = Command::new("clojure")
        .arg("-M")
        .arg("-e")
        .arg("(clojure.edn/read *in*)")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start Clojure Process");

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
        // .expect("Failed to write to stdin");
    }

    let mut output = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout
            .read_to_string(&mut output)
            .expect("Failed to read stdout");
    }

    let mut err = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        stderr
            .read_to_string(&mut err)
            .expect("Failed to read stderr");
    }
    err = err
        .strip_prefix(
            "Picked up _JAVA_OPTIONS: -Djava.util.prefs.userRoot=/home/user/.config/java\n",
        )
        .expect("Missing java stuff")
        .to_string();
    let mut msg = err.trim().split('\n');
    msg.next();
    err = msg.next().unwrap_or("").to_owned();
    if err.is_empty() {
        Ok(output)
    } else {
        Err((output, err))
    }
}

// {:country "GB", :gid #uuid "7d0162a9-2636-46f2-b0e8-a9336075eee2", :sort_name "His Master's Voice", :name "His Master's Voice", :type "Original Production"}
fn main() {
    let failing_files = vec![
        "examples/output/chat.edn",
        "examples/output/config.edn.2",
        "examples/output/create-access-token.edn",
        "examples/output/de.edn",
        "examples/output/de.edn.1",
        "examples/output/de.edn.2",
        "examples/output/default-rc.edn.1",
        "examples/output/dellstore-schema.edn",
        "examples/output/demo3.001.edn",
        "examples/output/double-ls.edn",
        "examples/output/double-ls.edn.1",
        "examples/output/double-ls.edn.2",
        "examples/output/double-ls.edn.3",
        "examples/output/double-ls.edn.4",
        "examples/output/example_list.edn",
        "examples/output/imperfect.edn",
        "examples/output/ja.edn",
        "examples/output/maps_unrecognized_keys.edn",
        "examples/output/number.edn",
        "examples/output/number.edn.1",
        "examples/output/number.edn.2",
        "examples/output/pipeline-with-includes.edn",
        "examples/output/pll_64M.edn",
        "examples/output/production.edn",
        "examples/output/provenance.edn",
        "examples/output/provenance.edn.1",
        "examples/output/provenance.edn.2",
        "examples/output/put-resource.edn",
        "examples/output/questions-2019.edn",
        "examples/output/RAM.edn",
        "examples/output/sample_data.edn",
        "examples/output/sample.edn.1",
        "examples/output/schema.edn.1",
        "examples/output/seattle-data1.edn",
        "examples/output/seattle-data1.edn.1",
        "examples/output/seattle-data1.edn.2",
        "examples/output/shadow-cljs.edn.17",
        "examples/output/shadow-cljs.edn.22",
        "examples/output/spfloat-comp.edn",
        "examples/output/spfloat-comp.edn.1",
        "examples/output/spfloat-comp.edn.2",
        "examples/output/spfloat-comp.edn.3",
        "examples/output/spfloat-comp.edn.4",
        "examples/output/system.edn",
        "examples/output/temperature.edn",
        "examples/output/time.edn",
        "examples/output/traefik.edn",
    ];
    for f in failing_files.into_iter() {
        let file = fs::read_to_string(f).expect("could not read file");
        if !is_match(file.as_str()) {
            println!("Failed on {f}")
        }
    }
}

#[cfg(test)]
mod tests {
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
            let file = fs::read_to_string(&path).unwrap();
            if !is_match(file.as_str()) {
                println!("{} does not match", path_str);
            }
        }
    }
}
