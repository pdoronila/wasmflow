// Common helper functions for extracting typed inputs
// Copy these functions into your component's src/lib.rs file as needed

fn extract_string_input(inputs: &[InputValue], name: &str) -> Result<String, ExecutionError> {
    let input = inputs.iter()
        .find(|i| i.name == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.value {
        NodeValue::String(s) => Ok(s.clone()),
        _ => Err(ExecutionError {
            message: format!("Expected string for input '{}', got {:?}", name, input.value),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a string value".to_string()),
        }),
    }
}

fn extract_u32_input(inputs: &[InputValue], name: &str) -> Result<u32, ExecutionError> {
    let input = inputs.iter()
        .find(|i| i.name == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.value {
        NodeValue::U32(n) => Ok(*n),
        _ => Err(ExecutionError {
            message: format!("Expected u32 for input '{}', got {:?}", name, input.value),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a positive integer value".to_string()),
        }),
    }
}

fn extract_f32_input(inputs: &[InputValue], name: &str) -> Result<f32, ExecutionError> {
    let input = inputs.iter()
        .find(|i| i.name == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.value {
        NodeValue::F32(n) => Ok(*n),
        _ => Err(ExecutionError {
            message: format!("Expected number for input '{}', got {:?}", name, input.value),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a numeric value".to_string()),
        }),
    }
}

fn extract_bool_input(inputs: &[InputValue], name: &str) -> Result<bool, ExecutionError> {
    let input = inputs.iter()
        .find(|i| i.name == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.value {
        NodeValue::Bool(b) => Ok(*b),
        _ => Err(ExecutionError {
            message: format!("Expected boolean for input '{}', got {:?}", name, input.value),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a boolean value (true/false)".to_string()),
        }),
    }
}
