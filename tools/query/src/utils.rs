use serde_json::Value;
use colored::Colorize;

pub fn colorize_json(json_str: &str) -> Result<String, serde_json::Error> {
    let v: Value = serde_json::from_str(json_str)?;
    let colored_json = colorize_value(&v, 0);
    Ok(colored_json)
}

fn colorize_value(value: &Value, indent: usize) -> String {
    match value {
        Value::Object(map) => {
            let mut entries = Vec::new();
            for (k, v) in map {
                let entry = format!(
                    "{}{}: {}",
                    " ".repeat(indent + 2),
                    k.blue(),
                    colorize_value(v, indent + 2)
                );
                entries.push(entry);
            }
            format!("{{\n{}\n{}}}", entries.join(",\n"), " ".repeat(indent))
        }
        Value::Array(arr) => {
            let elements: Vec<String> = arr
                .iter()
                .map(|v| format!("{}{}", " ".repeat(indent + 2), colorize_value(v, indent + 2)))
                .collect();
            format!("[\n{}\n{}]", elements.join(",\n"), " ".repeat(indent))
        }
        Value::String(s) => s.green().to_string(),
        Value::Number(n) => n.to_string().yellow().to_string(),
        Value::Bool(b) => {
            if *b {
                "true".cyan().to_string()
            } else {
                "false".red().to_string()
            }
        }
        Value::Null => "null".magenta().to_string(),
    }
}

