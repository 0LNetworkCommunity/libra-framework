use serde_json::Value;
use colored::Colorize;

pub fn colorize_and_print(json_str: &str) -> Result<(), serde_json::Error> {
    let v: Value = serde_json::from_str(json_str)?;
    let colored_json = colorize_value(&v, 0);
    println!("{}", colored_json);
    Ok(())
}

pub fn print_colored_kv(key: &str, value: &str) {
    let cleaned_value = if value.starts_with("\"") && value.ends_with("\"") {
        &value[1..value.len() - 1]
    } else {
        value
    };
    println!("{} : {}", key.magenta().bold(), cleaned_value.cyan());
}

fn colorize_value(value: &Value, indent: usize) -> String {
    match value {
        Value::Object(map) => {
            let mut entries = Vec::new();
            for (k, v) in map {
                let entry = format!(
                    "{}{}: {}",
                    " ".repeat(indent + 2),
                    k.magenta(),
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
        Value::String(s) => s.magenta().to_string(),  
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




