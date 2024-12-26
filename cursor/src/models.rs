use std::sync::LazyLock;
use crate::Model;

const MODEL_OBJECT: &str = "model";
const ANTHROPIC: &str = "anthropic";
const CURSOR: &str = "cursor";
const GOOGLE: &str = "google";
const OPENAI: &str = "openai";

pub static AVAILABLE_MODELS: LazyLock<Vec<Model>> = LazyLock::new(|| {
    vec![
        Model {
            id: "cursor-small".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: CURSOR.into()
        },
        Model {
            id: "claude-3-opus".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: ANTHROPIC.into()
        },
        Model {
            id: "cursor-fast".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: CURSOR.into()
        },
        Model {
            id: "gpt-3.5-turbo".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: OPENAI.into()
        },
        Model {
            id: "gpt-4-turbo-2024-04-09".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: OPENAI.into()
        },
        Model {
            id: "gpt-4".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: OPENAI.into()
        },
        Model {
            id: "gpt-4o-128k".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: OPENAI.into()
        },
        Model {
            id: "gemini-1.5-flash-500k".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: GOOGLE.into()
        },
        Model {
            id: "claude-3-haiku-200k".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: ANTHROPIC.into()
        },
        Model {
            id: "claude-3-5-sonnet-200k".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: ANTHROPIC.into()
        },
        Model {
            id: "claude-3-5-sonnet-20240620".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: ANTHROPIC.into()
        },
        Model {
            id: "claude-3-5-sonnet-20241022".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: ANTHROPIC.into()
        },
        Model {
            id: "gpt-4o-mini".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: OPENAI.into()
        },
        Model {
            id: "o1-mini".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: OPENAI.into()
        },
        Model {
            id: "o1-preview".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: OPENAI.into()
        },
        Model {
            id: "o1".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: OPENAI.into()
        },
        Model {
            id: "claude-3.5-haiku".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: ANTHROPIC.into()
        },
        Model {
            id: "gemini-exp-1206".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: GOOGLE.into()
        },
        Model {
            id: "gemini-2.0-flash-thinking-exp".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: GOOGLE.into()
        },
        Model {
            id: "gemini-2.0-flash-exp".into(),
            created: 1706659200,
            object: MODEL_OBJECT.into(),
            owned_by: GOOGLE.into()
        }
    ]
});