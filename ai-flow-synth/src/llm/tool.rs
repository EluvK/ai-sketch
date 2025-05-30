use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ToolFn = Box<dyn Fn(serde_json::Value) -> serde_json::Value + Send + Sync>;

pub struct ToolRegistry {
    pub map: HashMap<&'static str, (ToolFn, &'static str, serde_json::Value)>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register<T, F>(&mut self, name: &'static str, desc: &'static str, f: F)
    where
        T: for<'de> Deserialize<'de> + Serialize + JsonSchema + 'static,
        F: Fn(T) -> serde_json::Value + Send + Sync + 'static + Copy,
    {
        let schema = schemars::schema_for!(T);

        let wrapper: ToolFn = Box::new(move |value: serde_json::Value| {
            let input: T = serde_json::from_value(value).expect("Failed to deserialize input");
            f(input)
        });
        self.map.insert(name, (wrapper, desc, schema.to_value()));
    }

    pub fn get(&self, name: &str) -> Option<&(ToolFn, &'static str, serde_json::Value)> {
        self.map.get(name)
    }

    pub fn export_all_tools(&self) -> Vec<serde_json::Value> {
        self.map
            .iter()
            .map(|(name, (_func, desc, schema))| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": name,
                        "description": desc,
                        "parameters": schema,
                    }
                })
            })
            .collect()
    }

    pub fn export_tool(&self, name: &str) -> Option<serde_json::Value> {
        self.map.get(name).map(|(_func, desc, schema)| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": name,
                    "description": desc,
                    "parameters": schema,
                }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct GetWeatherParams {
        pub location: String,
    }

    fn get_weather(params: GetWeatherParams) -> serde_json::Value {
        // ...实际逻辑...
        serde_json::json!({"weather": "sunny", "location": params.location})
    }
    #[test]
    fn test_tool_schema() {
        let mut registry = ToolRegistry::new();
        registry.register::<GetWeatherParams, _>("get_weather", "获取天气", get_weather);

        // 假设 LLM 返回
        let func_name = "get_weather";
        let args = serde_json::json!({"location": "Hangzhou"});

        if let Some((func, _desc, _schema)) = registry.get(func_name) {
            let result = func(args);
            println!("调用结果: {:?}", result);
        }
    }
}
