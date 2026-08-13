use std::collections::HashMap;

pub fn run(arguments: impl IntoIterator<Item = String>) -> Result<u8, String> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    let Some(command) = arguments.first().map(String::as_str) else {
        print_help();
        return Ok(0);
    };

    if command == "help" || command == "--help" {
        print_help();
        return Ok(0);
    }

    let _options = Options::parse(&arguments[1..])?;
    Err(format!(
        "command '{command}' has not been migrated to the Rust CLI yet"
    ))
}

fn print_help() {
    println!("mc-loot-finder");
    println!("Rust migration build for Minecraft Java 26.1.2");
    println!();
    println!("Commands will be enabled as their results match the Java reference implementation.");
}

#[derive(Debug, Default, Eq, PartialEq)]
struct Options {
    values: HashMap<String, String>,
}

impl Options {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut values = HashMap::new();
        let mut index = 0;
        while index < arguments.len() {
            let key = &arguments[index];
            let Some(name) = key.strip_prefix("--") else {
                return Err(format!("expected an option, got: {key}"));
            };
            if name.is_empty() {
                return Err("option name must not be empty".to_owned());
            }
            if index + 1 == arguments.len() || arguments[index + 1].starts_with("--") {
                values.insert(name.to_owned(), "true".to_owned());
                index += 1;
            } else {
                values.insert(name.to_owned(), arguments[index + 1].clone());
                index += 2;
            }
        }
        Ok(Self { values })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_values_and_flags() {
        let arguments = ["--seed".to_owned(), "-1".to_owned(), "--json".to_owned()];
        let options = Options::parse(&arguments).unwrap();

        assert_eq!(options.values.get("seed").map(String::as_str), Some("-1"));
        assert_eq!(options.values.get("json").map(String::as_str), Some("true"));
    }

    #[test]
    fn rejects_positional_arguments() {
        let error = Options::parse(&["seed".to_owned()]).unwrap_err();
        assert_eq!(error, "expected an option, got: seed");
    }
}
