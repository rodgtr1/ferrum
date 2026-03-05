/// Pretty-print a JSON string. Returns the original string if parsing fails.
pub fn pretty_print(input: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(input) {
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| input.to_string())
    } else {
        input.to_string()
    }
}
