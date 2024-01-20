# rust-edn
Parser for Extensible Data Notation.

Goal to conform to the [EDN Spec](https://github.com/edn-format/edn) as close as possible

# TODO
- parse to a struct instead of raw pest parsing
- `#[derive(Serialize, Deserialize)]` traits
- handle #_, #inst, #uuid
- handle custom tagged elements

# Tested with examples from
https://github.com/shaunxcode/edn-tests  
https://github.com/antlr/grammars-v4/tree/master/edn  

# Other EDN libraries
https://crates.io/crates/edn-rs  
https://crates.io/crates/edn-format  
https://crates.io/crates/eden
