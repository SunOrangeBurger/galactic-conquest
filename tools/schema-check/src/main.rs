use contract::{schema, AgentResponse, SpectatorMap, SpectatorTick, TickRequest};
use serde_json::Value;
use std::{fs, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let outputs = [
        (
            "docs/schemas/tick-request.schema.json",
            schema::tick_request_schema(),
        ),
        (
            "docs/schemas/agent-response.schema.json",
            schema::agent_response_schema(),
        ),
        (
            "docs/schemas/spectator-map.schema.json",
            schema::spectator_map_schema(),
        ),
        (
            "docs/schemas/spectator-tick.schema.json",
            schema::spectator_tick_schema(),
        ),
    ];
    let write = std::env::args().any(|arg| arg == "--write");
    for (relative, value) in outputs {
        let path = root.join(relative);
        let serialized = format!("{}\n", serde_json::to_string_pretty(&value).unwrap());
        if write {
            if let Some(parent) = path.parent() {
                if let Err(error) = fs::create_dir_all(parent) {
                    return fail(format!("cannot create {}: {error}", parent.display()));
                }
            }
            if let Err(error) = fs::write(&path, serialized) {
                return fail(format!("cannot write {}: {error}", path.display()));
            }
        } else if fs::read_to_string(&path).ok().as_deref() != Some(serialized.as_str()) {
            return fail(format!(
                "{} is stale; run cargo run -p schema-check -- --write",
                path.display()
            ));
        }
    }

    let examples = [
        ("docs/examples/tick-request.json", ExampleKind::TickRequest),
        (
            "docs/examples/agent-response.json",
            ExampleKind::AgentResponse,
        ),
        (
            "docs/examples/spectator-map.json",
            ExampleKind::SpectatorMap,
        ),
        (
            "docs/examples/spectator-tick.json",
            ExampleKind::SpectatorTick,
        ),
    ];
    for (relative, kind) in examples {
        let text = match fs::read_to_string(root.join(relative)) {
            Ok(text) => text,
            Err(error) => return fail(format!("cannot read {relative}: {error}")),
        };
        let value: Value = match serde_json::from_str(&text) {
            Ok(value) => value,
            Err(error) => return fail(format!("invalid JSON in {relative}: {error}")),
        };
        let result = match kind {
            ExampleKind::TickRequest => validate_schema(&value, &schema::tick_request_schema())
                .and_then(|()| {
                    serde_json::from_value::<TickRequest>(value)
                        .map(|_| ())
                        .map_err(|e| e.to_string())
                }),
            ExampleKind::AgentResponse => validate_schema(&value, &schema::agent_response_schema())
                .and_then(|()| {
                    serde_json::from_value::<AgentResponse>(value)
                        .map(|_| ())
                        .map_err(|e| e.to_string())
                }),
            ExampleKind::SpectatorMap => validate_schema(&value, &schema::spectator_map_schema())
                .and_then(|()| {
                    serde_json::from_value::<SpectatorMap>(value)
                        .map(|_| ())
                        .map_err(|e| e.to_string())
                }),
            ExampleKind::SpectatorTick => validate_schema(&value, &schema::spectator_tick_schema())
                .and_then(|()| {
                    serde_json::from_value::<SpectatorTick>(value)
                        .map(|_| ())
                        .map_err(|e| e.to_string())
                }),
        };
        if let Err(error) = result {
            return fail(format!(
                "{relative} fails schema/contract validation: {error}"
            ));
        }
    }
    println!("JSON Schemas match the contract and all documented examples validate");
    ExitCode::SUCCESS
}

#[derive(Clone, Copy)]
enum ExampleKind {
    TickRequest,
    AgentResponse,
    SpectatorMap,
    SpectatorTick,
}

fn validate_schema(value: &Value, schema: &Value) -> Result<(), String> {
    validate(value, schema, schema, "$")
}

fn validate(value: &Value, spec: &Value, root: &Value, path: &str) -> Result<(), String> {
    if let Some(reference) = spec.get("$ref").and_then(Value::as_str) {
        let target = reference
            .strip_prefix("#/$defs/")
            .ok_or_else(|| format!("unsupported schema reference {reference}"))?;
        return validate(value, &root["$defs"][target], root, path);
    }
    if let Some(expected) = spec.get("const") {
        if value != expected {
            return Err(format!("{path}: expected {expected}"));
        }
    }
    if let Some(values) = spec.get("enum").and_then(Value::as_array) {
        if !values.contains(value) {
            return Err(format!("{path}: value is outside enum"));
        }
    }
    if let Some(types) = spec.get("type") {
        let valid = if let Some(types) = types.as_array() {
            types
                .iter()
                .any(|t| matches_type(value, t.as_str().unwrap_or("")))
        } else {
            matches_type(value, types.as_str().unwrap_or(""))
        };
        if !valid {
            return Err(format!("{path}: wrong JSON type"));
        }
    }
    if let Some(number) = value.as_f64() {
        if spec
            .get("minimum")
            .and_then(Value::as_f64)
            .is_some_and(|minimum| number < minimum)
        {
            return Err(format!("{path}: number is below minimum"));
        }
        if spec
            .get("maximum")
            .and_then(Value::as_f64)
            .is_some_and(|maximum| number > maximum)
        {
            return Err(format!("{path}: number is above maximum"));
        }
    }
    if let Some(array) = value.as_array() {
        if spec
            .get("minItems")
            .and_then(Value::as_u64)
            .is_some_and(|n| array.len() < n as usize)
        {
            return Err(format!("{path}: array has too few items"));
        }
        if spec
            .get("maxItems")
            .and_then(Value::as_u64)
            .is_some_and(|n| array.len() > n as usize)
        {
            return Err(format!("{path}: array has too many items"));
        }
    }
    if let Some(required) = spec.get("required").and_then(Value::as_array) {
        for key in required.iter().filter_map(Value::as_str) {
            if value.get(key).is_none() {
                return Err(format!("{path}: missing required field {key}"));
            }
        }
    }
    if let Some(properties) = spec.get("properties").and_then(Value::as_object) {
        if let Some(object) = value.as_object() {
            if spec.get("additionalProperties") == Some(&Value::Bool(false)) {
                if let Some(extra) = object.keys().find(|key| !properties.contains_key(*key)) {
                    return Err(format!("{path}: unexpected field {extra}"));
                }
            }
            for (key, child_spec) in properties {
                if let Some(child) = object.get(key) {
                    validate(child, child_spec, root, &format!("{path}.{key}"))?;
                }
            }
            if let Some(extra_spec) = spec
                .get("additionalProperties")
                .filter(|value| value.is_object())
            {
                for (key, child) in object {
                    if !properties.contains_key(key) {
                        validate(child, extra_spec, root, &format!("{path}.{key}"))?;
                    }
                }
            }
        }
    }
    if let (Some(items), Some(array)) = (spec.get("items"), value.as_array()) {
        for (index, item) in array.iter().enumerate() {
            validate(item, items, root, &format!("{path}[{index}]"))?;
        }
    }
    if let Some(choices) = spec.get("oneOf").and_then(Value::as_array) {
        let passing = choices
            .iter()
            .filter(|choice| validate(value, choice, root, path).is_ok())
            .count();
        if passing != 1 {
            return Err(format!(
                "{path}: expected exactly one matching schema, got {passing}"
            ));
        }
    }
    Ok(())
}

fn matches_type(value: &Value, expected: &str) -> bool {
    match expected {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => false,
    }
}

fn fail(message: String) -> ExitCode {
    eprintln!("{message}");
    ExitCode::FAILURE
}
