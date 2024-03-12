use serde_json::Value;

pub fn pop_first_element(root: &Value) -> Value {
    if let Some(array) = root.as_array() {
        let mut trimmed = array.clone();
        trimmed.remove(0);
        return trimmed.iter().map(|x| {
            pop_first_element(x)
        }).collect();
    } else {
        return root.clone();
    }
}