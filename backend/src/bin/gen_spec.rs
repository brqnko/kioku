fn main() {
    let doc = backend::server::api_doc();
    let yaml = doc.to_yaml().unwrap();
    print!("{}", required_to_flow(&yaml));
}

fn required_to_flow(yaml: &str) -> String {
    let mut result = String::new();
    let mut lines = yaml.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim_end();
        // "required:" のみの行（"required: true" 等は除外）
        if trimmed.ends_with("required:") && !trimmed.contains("required: ") {
            let indent = &line[..line.len() - line.trim_start().len()];
            let item_prefix = format!("{}- ", indent);
            let mut items = Vec::new();
            while lines
                .peek()
                .map_or(false, |next| next.starts_with(&item_prefix))
            {
                let item_line = lines.next().unwrap();
                items.push(item_line.trim().trim_start_matches("- ").to_string());
            }
            if !items.is_empty() {
                result.push_str(&format!("{}required: [{}]\n", indent, items.join(", ")));
            } else {
                result.push_str(line);
                result.push('\n');
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}
