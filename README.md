# rust-edn
Parser for Extensible Data Notation.

Goal to conform to the [EDN Spec](https://github.com/edn-format/edn) as close as possible

# TODO
- `#[derive(Serialize, Deserialize)]` traits
<!-- - to string
- publish to crates.io
-
- handle #_, #inst, #uuid
- diff fuzz

# Tested with examples from
https://github.com/shaunxcode/edn-tests
https://github.com/antlr/grammars-v4/tree/master/edn

# Other EDN libraries
https://crates.io/crates/edn-rs
https://crates.io/crates/edn-format
https://crates.io/crates/eden

https://github.com/riscv/riscv-isa-manual/tree/70040578316b9978056c9f33ac654ea19f459169/src/images/wavedrom
WaveDROM https://wavedrom.com/ uses edn

$ gh search code --extension "edn" --json "repository,url" --limit 800 > output.json
$ jq -r '.[].url' < ../output.json | sed -e "s/github/raw.githubusercontent/" | sed -e "s/\/blob//" | xargs -- wget {}

https://github.com/OSI-INC/P3051/blob/master/RAM.edn
RAM.edn is EDIF apparently
https://en.wikipedia.org/wiki/EDIF

https://nitor.com/en/articles/pitfalls-and-bumps-clojures-extensible-data-notation-edn -->
