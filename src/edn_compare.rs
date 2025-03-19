use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

use crate::edn_reader;

pub fn rust_edn(input: &str) -> Result<String, String> {
    let a = edn_reader::read_str(input.to_string())?;
    let b = format!("{a}");
    Ok(b)
}

pub fn clojure_edn(input: &str) -> Result<String, String> {
    let mut child = Command::new("clojure")
        .arg("-M")
        .arg("-e")
        .arg("(do (set! *print-namespace-maps* false) (clojure.edn/read *in*))")
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
        Ok(output.trim().to_string())
    } else {
        Err(format!("{output}{err}").trim().to_string())
    }
}
