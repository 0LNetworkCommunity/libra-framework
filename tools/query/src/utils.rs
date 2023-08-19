use anyhow::Context;
use colored::Colorize;
use serde_json::Value;

pub fn colorize_and_print(json_str: &str) -> anyhow::Result<()> {
    let v: Value = serde_json::from_str(json_str).context("Failed to parse JSON string")?;
    let colored_json = colorize_value(&v, 0);
    println!("{}", colored_json);
    Ok(())
}

pub fn print_colored_kv(key: &str, value: &str) {
    let cleaned_value = if value.starts_with('\"') && value.ends_with('\"') {
        &value[1..value.len() - 1]
    } else {
        value
    };
    println!("{} : {}", key.magenta().bold(), cleaned_value.cyan());
}

pub fn format_colored_kv(key: &str, value: &str) -> String {
    format!("{}: {}", key.bright_yellow(), value.bright_green())
}

fn colorize_value(value: &Value, indent: usize) -> String {
    match value {
        Value::Object(map) => {
            let mut entries = Vec::new();
            for (k, v) in map {
                let entry = format!(
                    "{}{}: {}",
                    " ".repeat(indent + 2),
                    k.purple(),
                    colorize_value(v, indent + 2)
                );
                entries.push(entry);
            }
            format!("{{\n{}\n{}}}", entries.join(",\n"), " ".repeat(indent))
        }
        Value::Array(arr) => {
            let elements: Vec<String> = arr
                .iter()
                .map(|v| {
                    format!(
                        "{}{}",
                        " ".repeat(indent + 2),
                        colorize_value(v, indent + 2)
                    )
                })
                .collect();
            format!("[\n{}\n{}]", elements.join(",\n"), " ".repeat(indent))
        }
        Value::String(s) => s.cyan().to_string(),
        Value::Number(n) => n.to_string().cyan().to_string(),
        Value::Bool(b) => {
            if *b {
                "true".cyan().to_string()
            } else {
                "false".red().to_string()
            }
        }
        Value::Null => "null".cyan().to_string(),
    }
}
