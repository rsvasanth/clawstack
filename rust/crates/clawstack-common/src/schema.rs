//! JSON Schema exports — used for code generation in Python and validation.

/// Returns the JSON Schema for AgentRequest.
pub fn agent_request_schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "AgentRequest",
        "type": "object",
        "properties": {
            "workspace_id": { "type": "string", "format": "uuid" },
            "session_id": { "type": "string", "format": "uuid" },
            "user_id": { "type": "string", "format": "uuid" },
            "channel": { "type": "string" },
            "text": { "type": "string" },
            "attachments": { "type": "array", "items": { "$ref": "#/definitions/Attachment" } },
            "metadata": { "type": "object" }
        },
        "required": ["workspace_id", "user_id", "text"]
    })
}

/// Returns the JSON Schema for ToolSpec.
pub fn tool_spec_schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "ToolSpec",
        "type": "object",
        "properties": {
            "id": { "type": "string" },
            "description": { "type": "string" },
            "input_schema": { "type": "object" },
            "output_schema": { "type": "object" },
            "required_capabilities": { "type": "array", "items": { "type": "string" } },
            "execution_mode": { "type": "string", "enum": ["sync", "async", "worker"] },
            "timeout_ms": { "type": "integer", "minimum": 0 },
            "side_effect_class": { "type": "string", "enum": ["none", "read", "write", "exec", "admin"] }
        },
        "required": ["id", "description", "input_schema", "output_schema"]
    })
}

/// Returns the JSON Schema for Skill.
pub fn skill_schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Skill",
        "type": "object",
        "properties": {
            "id": { "type": "string" },
            "name": { "type": "string" },
            "description": { "type": "string" },
            "version": { "type": "integer", "minimum": 1 },
            "scope": { "type": "string", "enum": ["coding", "data", "messaging", "web", "system", "general"] },
            "inputs": { "type": "array", "items": { "$ref": "#/definitions/SkillInput" } },
            "steps": { "type": "array", "items": { "type": "string" } },
            "success_signals": { "type": "array", "items": { "type": "string" } },
            "risk_level": { "type": "string", "enum": ["none", "low", "medium", "high", "critical"] },
            "required_capabilities": { "type": "array", "items": { "type": "string" } },
            "source": { "type": "string", "enum": ["builtin", "installed", "learned", "wasm"] },
            "enabled": { "type": "boolean" }
        },
        "required": ["id", "name", "description", "version", "scope"]
    })
}

pub fn agent_response_schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "AgentResponse",
        "type": "object",
        "properties": {
            "text": { "type": "string" },
            "attachments": { "type": "array" },
            "tool_calls": { "type": "array" },
            "tool_results": { "type": "array" },
            "session_id": { "type": "string" }
        }
    })
}
