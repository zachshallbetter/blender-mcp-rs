use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use schemars::{schema_for, JsonSchema};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Tool '{0}' not found in registry")]
    NotFound(String),

    #[error("Failed to parse tool parameters: {0}")]
    InvalidParameters(#[from] serde_json::Error),

    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Internal tool error: {0}")]
    Internal(String),
}

/// Metadata description for an introspectable MCP tool.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Trait implemented by introspectable, reflective tools.
#[async_trait]
pub trait ReflectTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, ToolError>;
}

/// Adapter converting any async function with typed input `P` and output `R` into a reflective MCP tool.
pub struct TypedTool<P, R, F, Fut>
where
    P: JsonSchema + DeserializeOwned + Send + 'static,
    R: Serialize + Send + 'static,
    F: Fn(P) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<R, String>> + Send + 'static,
{
    name: String,
    description: String,
    handler: F,
    _marker: PhantomData<(P, R)>,
}

impl<P, R, F, Fut> TypedTool<P, R, F, Fut>
where
    P: JsonSchema + DeserializeOwned + Send + 'static,
    R: Serialize + Send + 'static,
    F: Fn(P) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<R, String>> + Send + 'static,
{
    pub fn new(name: impl Into<String>, description: impl Into<String>, handler: F) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            handler,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<P, R, F, Fut> ReflectTool for TypedTool<P, R, F, Fut>
where
    P: JsonSchema + DeserializeOwned + Send + Sync + 'static,
    R: Serialize + Send + Sync + 'static,
    F: Fn(P) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<R, String>> + Send + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> serde_json::Value {
        let schema = schema_for!(P);
        serde_json::to_value(schema).unwrap_or(serde_json::json!({
            "type": "object"
        }))
    }

    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, ToolError> {
        let parsed: P = serde_json::from_value(params)?;
        let result = (self.handler)(parsed)
            .await
            .map_err(ToolError::ExecutionFailed)?;
        let serialized = serde_json::to_value(result)?;
        Ok(serialized)
    }
}

/// Central reflective registry managing MCP tools and runtime introspection.
#[derive(Default, Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn ReflectTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: impl ReflectTool + 'static) {
        let name = tool.name().to_string();
        self.tools.insert(name, Arc::new(tool));
    }

    pub fn register_arc(&mut self, tool: Arc<dyn ReflectTool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    pub fn register_fn<P, R, F, Fut>(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        handler: F,
    ) where
        P: JsonSchema + DeserializeOwned + Send + Sync + 'static,
        R: Serialize + Send + Sync + 'static,
        F: Fn(P) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, String>> + Send + 'static,
    {
        let tool = TypedTool::new(name, description, handler);
        self.register(tool);
    }

    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|t| ToolDefinition {
                name: t.name().to_string(),
                description: t.description().to_string(),
                input_schema: t.input_schema(),
            })
            .collect()
    }

    pub async fn dispatch(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ToolError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;
        tool.execute(params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize, JsonSchema)]
    struct AddParams {
        /// The first number
        a: i32,
        /// The second number
        b: i32,
    }

    #[derive(Serialize)]
    struct AddResult {
        sum: i32,
    }

    #[tokio::test]
    async fn test_reflective_tool_registry() {
        let mut registry = ToolRegistry::new();

        registry.register_fn(
            "add_numbers",
            "Adds two integers together",
            |params: AddParams| async move { Ok(AddResult { sum: params.a + params.b }) },
        );

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "add_numbers");
        assert!(tools[0].description.contains("Adds two integers"));

        // Verify JSON schema was generated automatically via reflection
        let schema_obj = tools[0].input_schema.as_object().unwrap();
        assert!(schema_obj.contains_key("properties"));

        // Dispatch call
        let result = registry
            .dispatch("add_numbers", serde_json::json!({ "a": 5, "b": 7 }))
            .await
            .unwrap();

        assert_eq!(result, serde_json::json!({ "sum": 12 }));
    }
}
