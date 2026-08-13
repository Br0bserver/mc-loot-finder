use std::collections::HashMap;

use crate::rust_core::candidate_structure;
use crate::rust_core::candidates::locate;

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

    let options = Options::parse(&arguments[1..])?;
    match command {
        "candidates" => candidates(&options),
        _ => Err(format!(
            "command '{command}' has not been migrated to the Rust CLI yet"
        )),
    }
}

fn print_help() {
    println!("mc-loot-finder");
    println!("Rust migration build for Minecraft Java 26.1.2");
    println!();
    println!("Commands will be enabled as their results match the Java reference implementation.");
}

fn candidates(options: &Options) -> Result<u8, String> {
    require_version(options)?;
    let structure = candidate_structure(options.text("structure", "ancient_city"))?;
    let seed = options.required_i64("seed")?;
    let center_x = options.i32("center-x", 0)?;
    let center_z = options.i32("center-z", 0)?;
    let radius = options.i32("radius", 5_000)?;
    let limit = options.i32("limit", 100)?;
    if limit < 0 {
        return Err("--limit must be non-negative".to_owned());
    }
    let candidates = locate(seed, center_x, center_z, radius, structure.placement)?;

    if options.flag("json") {
        print!(
            "{{\"version\":\"26.1.2\",\"structure\":\"{}\",\"seed\":{},\"status\":\"candidate_only\",\"candidates\":[",
            structure.name, seed
        );
        for (index, candidate) in candidates.iter().take(limit as usize).enumerate() {
            if index != 0 {
                print!(",");
            }
            print!(
                "{{\"chunk_x\":{},\"chunk_z\":{},\"block_x\":{},\"block_z\":{},\"distance\":{:.3}}}",
                candidate.chunk_x,
                candidate.chunk_z,
                candidate.block_x,
                candidate.block_z,
                (candidate.squared_distance as f64).sqrt()
            );
        }
        println!("]}}");
        return Ok(0);
    }

    println!("Minecraft Java 26.1.2");
    println!("World seed: {seed}");
    println!("Structure: {}", structure.name);
    println!(
        "Search area: {} blocks around ({center_x}, {center_z})\n",
        grouped(i64::from(radius))
    );
    println!(
        "Found {}\n",
        quantity(candidates.len() as i64, "placement candidate")
    );
    let shown = candidates.len().min(limit as usize);
    for (index, candidate) in candidates.iter().take(shown).enumerate() {
        println!("[{}]", index + 1);
        println!("  Chunk: ({}, {})", candidate.chunk_x, candidate.chunk_z);
        println!("  Center: ({}, {})", candidate.block_x, candidate.block_z);
        println!(
            "  Distance: {:.1} blocks\n",
            (candidate.squared_distance as f64).sqrt()
        );
    }
    println!("Candidates are not verified structures.");
    println!("Use 'chests' or 'find' to verify them.");
    println!(
        "Shown: {} of {}",
        grouped(shown as i64),
        quantity(candidates.len() as i64, "placement candidate")
    );
    Ok(0)
}

fn require_version(options: &Options) -> Result<(), String> {
    let version = options.text("version", "26.1.2");
    if version == "26.1.2" {
        Ok(())
    } else {
        Err(format!(
            "unsupported Minecraft version: {version}; supported: 26.1.2"
        ))
    }
}

fn grouped(value: i64) -> String {
    let negative = value < 0;
    let digits = value.unsigned_abs().to_string();
    let mut result = String::with_capacity(digits.len() + digits.len() / 3 + usize::from(negative));
    if negative {
        result.push('-');
    }
    for (index, character) in digits.chars().enumerate() {
        if index != 0 && (digits.len() - index).is_multiple_of(3) {
            result.push(',');
        }
        result.push(character);
    }
    result
}

fn quantity(count: i64, singular: &str) -> String {
    format!(
        "{} {singular}{}",
        grouped(count),
        if count == 1 { "" } else { "s" }
    )
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

    fn text<'a>(&'a self, key: &str, fallback: &'a str) -> &'a str {
        self.values.get(key).map_or(fallback, String::as_str)
    }

    fn required_i64(&self, key: &str) -> Result<i64, String> {
        let value = self
            .values
            .get(key)
            .ok_or_else(|| format!("missing required option --{key}"))?;
        value
            .parse()
            .map_err(|_| format!("--{key} must be a 64-bit integer"))
    }

    fn i32(&self, key: &str, fallback: i32) -> Result<i32, String> {
        let Some(value) = self.values.get(key) else {
            return Ok(fallback);
        };
        value
            .parse()
            .map_err(|_| format!("--{key} must be a 32-bit integer"))
    }

    fn flag(&self, key: &str) -> bool {
        self.values.get(key).is_some_and(|value| value == "true")
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

    #[test]
    fn formats_grouped_numbers() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(5_000), "5,000");
        assert_eq!(grouped(-1_234_567), "-1,234,567");
    }
}
