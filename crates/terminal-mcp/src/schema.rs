//! JSON Schema transformation for AI client compatibility.
//!
//! This module transforms schemars-generated JSON Schema (draft-2020-12) into
//! formats compatible with various AI clients that only support draft-07.
//!
//! Transformations include:
//! - `$defs` → `definitions` (draft-07 compatibility)
//! - `anyOf` with nullable → simpler nullable patterns
//! - Reference resolution and simplification

use serde_json::Value;

/// Schema transformer for AI client compatibility.
pub struct SchemaTransformer;

impl SchemaTransformer {
    /// Transform a JSON Schema for maximum AI client compatibility.
    ///
    /// Applies transformations in sequence:
    /// 1. Convert `$defs` to `definitions`
    /// 2. Simplify `anyOf` with nullable patterns
    /// 3. Update references to use `definitions`
    pub fn transform(mut schema: Value) -> Value {
        schema = Self::convert_defs_to_definitions(schema);
        schema = Self::simplify_nullable_anyof(schema);
        schema
    }

    /// Convert `$defs` to `definitions` for draft-07 compatibility.
    ///
    /// Draft-2020-12 uses `$defs`, but draft-07 uses `definitions`.
    /// This also updates all references from `#/$defs/` to `#/definitions/`.
    fn convert_defs_to_definitions(mut schema: Value) -> Value {
        if let Some(obj) = schema.as_object_mut() {
            // Move $defs to definitions
            if let Some(defs) = obj.remove("$defs") {
                obj.insert("definitions".to_string(), defs);
            }

            // Recursively update references
            Self::update_references(obj);
        }
        schema
    }

    /// Update all `$ref` values from `#/$defs/` to `#/definitions/`.
    fn update_references(value: &mut serde_json::Map<String, Value>) {
        for (key, val) in value.iter_mut() {
            if key == "$ref" {
                if let Some(ref_str) = val.as_str() {
                    if ref_str.starts_with("#/$defs/") {
                        *val = Value::String(ref_str.replace("#/$defs/", "#/definitions/"));
                    }
                }
            } else {
                // Recurse into nested objects and arrays
                match val {
                    Value::Object(obj) => Self::update_references(obj),
                    Value::Array(arr) => {
                        for item in arr {
                            if let Value::Object(obj) = item {
                                Self::update_references(obj);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Simplify `anyOf` patterns with nullable.
    ///
    /// Converts patterns like `{"anyOf": [{"type": "string"}, {"type": "null"}]}`
    /// to simpler `{"type": "string", "nullable": true}` (or removes if not needed).
    fn simplify_nullable_anyof(mut schema: Value) -> Value {
        if let Some(obj) = schema.as_object_mut() {
            Self::simplify_anyof_in_object(obj);
        }
        schema
    }

    /// Recursively simplify anyOf patterns in an object.
    fn simplify_anyof_in_object(obj: &mut serde_json::Map<String, Value>) {
        // Check if this object has an anyOf pattern
        if let Some(Value::Array(any_of)) = obj.get("anyOf") {
            if let Some(simplified) = Self::try_simplify_anyof(any_of) {
                // Remove anyOf and apply simplified version
                obj.remove("anyOf");
                for (k, v) in simplified.as_object().unwrap() {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }

        // Recurse into all nested objects
        for val in obj.values_mut() {
            match val {
                Value::Object(nested) => Self::simplify_anyof_in_object(nested),
                Value::Array(arr) => {
                    for item in arr {
                        if let Value::Object(nested) = item {
                            Self::simplify_anyof_in_object(nested);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Try to simplify an anyOf array if it's a nullable pattern.
    ///
    /// Returns Some(simplified_schema) if simplification is possible, None otherwise.
    fn try_simplify_anyof(any_of: &[Value]) -> Option<Value> {
        // Look for pattern: [{type: X}, {type: "null"}] or [{$ref: Y}, {type: "null"}]
        if any_of.len() != 2 {
            return None;
        }

        let (type_schema, null_schema) = if Self::is_null_type(&any_of[1]) {
            (&any_of[0], &any_of[1])
        } else if Self::is_null_type(&any_of[0]) {
            (&any_of[1], &any_of[0])
        } else {
            return None;
        };

        // Don't add nullable if the null type has no purpose
        // Just return the non-null type
        if null_schema.as_object().map_or(false, |o| o.len() == 1) {
            Some(type_schema.clone())
        } else {
            None
        }
    }

    /// Check if a schema represents the null type.
    fn is_null_type(schema: &Value) -> bool {
        schema
            .as_object()
            .and_then(|o| o.get("type"))
            .and_then(|t| t.as_str())
            .map_or(false, |t| t == "null")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_convert_defs_to_definitions() {
        let schema = json!({
            "$defs": {
                "MyType": {
                    "type": "string"
                }
            },
            "properties": {
                "field": {
                    "$ref": "#/$defs/MyType"
                }
            }
        });

        let result = SchemaTransformer::convert_defs_to_definitions(schema);

        assert!(result["$defs"].is_null());
        assert!(result["definitions"]["MyType"].is_object());
        assert_eq!(
            result["properties"]["field"]["$ref"],
            "#/definitions/MyType"
        );
    }

    #[test]
    fn test_simplify_nullable_anyof() {
        let schema = json!({
            "anyOf": [
                {"type": "string"},
                {"type": "null"}
            ]
        });

        let result = SchemaTransformer::simplify_nullable_anyof(schema);

        assert!(result["anyOf"].is_null());
        assert_eq!(result["type"], "string");
    }

    #[test]
    fn test_transform_full() {
        let schema = json!({
            "$defs": {
                "Dimensions": {
                    "type": "object",
                    "properties": {
                        "rows": {"type": "integer"},
                        "cols": {"type": "integer"}
                    }
                }
            },
            "properties": {
                "dimensions": {
                    "$ref": "#/$defs/Dimensions"
                },
                "optional": {
                    "anyOf": [
                        {"type": "string"},
                        {"type": "null"}
                    ]
                }
            }
        });

        let result = SchemaTransformer::transform(schema);

        // Check $defs → definitions
        assert!(result["$defs"].is_null());
        assert!(result["definitions"]["Dimensions"].is_object());

        // Check ref updated
        assert_eq!(
            result["properties"]["dimensions"]["$ref"],
            "#/definitions/Dimensions"
        );

        // Check anyOf simplified
        assert!(result["properties"]["optional"]["anyOf"].is_null());
        assert_eq!(result["properties"]["optional"]["type"], "string");
    }
}
