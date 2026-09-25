//! Shared contract types for the pulse-null plugin ecosystem.
//!
//! This crate defines the types that plugin crates and pulse-null both depend on.
//! It contains no logic — just structs, enums, and trait-adjacent types that form
//! the contract between the orchestrator and its plugins.

pub mod llm;
pub mod monitoring;
pub mod plugin;
pub mod tool;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Plugin health status, returned by plugin `health()` methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "message")]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Down(String),
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded(msg) => write!(f, "degraded: {msg}"),
            Self::Down(msg) => write!(f, "down: {msg}"),
        }
    }
}

/// Plugin metadata — identity and version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub description: String,
}

/// A setup prompt for the init wizard.
///
/// Plugins return a list of these to tell pulse-null what configuration
/// values they need during first-time setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupPrompt {
    /// Config key (e.g. "twilio_account_sid")
    pub key: String,
    /// Human-readable question (e.g. "Twilio Account SID:")
    pub question: String,
    /// Default value if the user presses enter
    pub default: Option<String>,
    /// Whether a value is required
    pub required: bool,
    /// Whether the value is sensitive (mask input)
    pub secret: bool,
}

/// A scheduled task definition that plugins can contribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    /// Cron expression (6-field: sec min hour dom month dow)
    pub cron: String,
    /// Trust channel for the session
    pub channel: String,
    /// Prompt sent to the LLM
    pub prompt: String,
    /// How to route the output
    #[serde(default)]
    pub output_routing: OutputRouting,
    /// Whether this task is active
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Who created this task
    #[serde(default)]
    pub created_by: TaskCreator,
    /// Optional evaluator type — if set, the scheduler checks preconditions
    /// before firing. Currently supported: "pipeline" (checks if pipeline
    /// documents have changed since last fire).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluator: Option<String>,
}

fn default_true() -> bool {
    true
}

/// How to route a scheduled task's output.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputRouting {
    #[default]
    Silent,
    Share,
    Call,
}

/// Who created the task.
///
/// Serialized in lowercase (`"system"`, `"pulse"`, `"user"`). The legacy
/// `"entity"` value written before 0.7.0 still deserializes as
/// [`TaskCreator::Pulse`].
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskCreator {
    /// Built into the runtime or contributed by a plugin.
    #[default]
    System,
    /// Created by the pulse itself at runtime.
    #[serde(alias = "entity")]
    Pulse,
    /// Created by a human operator.
    User,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(
            HealthStatus::Degraded("high latency".into()).to_string(),
            "degraded: high latency"
        );
        assert_eq!(
            HealthStatus::Down("connection refused".into()).to_string(),
            "down: connection refused"
        );
    }

    #[test]
    fn health_status_serializes() {
        let json = serde_json::to_string(&HealthStatus::Healthy).unwrap();
        assert!(json.contains("Healthy"));

        let json = serde_json::to_string(&HealthStatus::Degraded("slow".into())).unwrap();
        assert!(json.contains("Degraded"));
        assert!(json.contains("slow"));
    }

    #[test]
    fn plugin_meta_roundtrip() {
        let meta = PluginMeta {
            name: "voice-echo".into(),
            version: "0.7.0".into(),
            description: "Phone calls via Twilio".into(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let back: PluginMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "voice-echo");
        assert_eq!(back.version, "0.7.0");
    }

    #[test]
    fn setup_prompt_with_defaults() {
        let prompt = SetupPrompt {
            key: "host".into(),
            question: "Bind address:".into(),
            default: Some("0.0.0.0".into()),
            required: false,
            secret: false,
        };
        assert_eq!(prompt.default.as_deref(), Some("0.0.0.0"));
        assert!(!prompt.secret);
    }

    #[test]
    fn setup_prompt_secret() {
        let prompt = SetupPrompt {
            key: "api_key".into(),
            question: "API Key:".into(),
            default: None,
            required: true,
            secret: true,
        };
        assert!(prompt.secret);
        assert!(prompt.required);
        assert!(prompt.default.is_none());
    }

    #[test]
    fn scheduled_task_defaults() {
        let json = r#"{
            "id": "morning",
            "name": "Morning check",
            "cron": "0 0 8 * * *",
            "channel": "reflection",
            "prompt": "Good morning."
        }"#;
        let task: ScheduledTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.output_routing, OutputRouting::Silent);
        assert!(task.enabled);
        assert_eq!(task.created_by, TaskCreator::System);
        assert!(task.evaluator.is_none());
    }

    #[test]
    fn scheduled_task_with_evaluator() {
        let json = r#"{
            "id": "reflection",
            "name": "Reflection Window",
            "cron": "0 30 4 * * *",
            "channel": "system",
            "prompt": "Reflect.",
            "evaluator": "pipeline"
        }"#;
        let task: ScheduledTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.evaluator.as_deref(), Some("pipeline"));
    }

    #[test]
    fn task_creator_pulse_serializes_as_pulse() {
        let json = serde_json::to_string(&TaskCreator::Pulse).unwrap();
        assert_eq!(json, r#""pulse""#);
        let back: TaskCreator = serde_json::from_str(&json).unwrap();
        assert_eq!(back, TaskCreator::Pulse);
    }

    #[test]
    fn task_creator_legacy_entity_deserializes_as_pulse() {
        let creator: TaskCreator = serde_json::from_str(r#""entity""#).unwrap();
        assert_eq!(creator, TaskCreator::Pulse);
    }

    #[test]
    fn task_creator_rejects_unknown_value() {
        assert!(serde_json::from_str::<TaskCreator>(r#""robot""#).is_err());
    }

    #[test]
    fn scheduled_task_legacy_entity_creator_loads_and_rewrites_as_pulse() {
        let legacy = r#"{
            "id": "self-made",
            "name": "Self-made task",
            "cron": "0 0 9 * * *",
            "channel": "reflection",
            "prompt": "Check in.",
            "created_by": "entity"
        }"#;
        let task: ScheduledTask = serde_json::from_str(legacy).unwrap();
        assert_eq!(task.created_by, TaskCreator::Pulse);

        let rewritten: serde_json::Value = serde_json::to_value(&task).unwrap();
        assert_eq!(rewritten["created_by"], "pulse");

        let back: ScheduledTask = serde_json::from_value(rewritten).unwrap();
        assert_eq!(back.created_by, TaskCreator::Pulse);
    }

    #[test]
    fn output_routing_variants() {
        let json = r#""share""#;
        let routing: OutputRouting = serde_json::from_str(json).unwrap();
        assert_eq!(routing, OutputRouting::Share);
    }
}
