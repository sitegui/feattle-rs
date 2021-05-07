///! Describes the schema of the JSON API
use feattle_core::last_reload::LastReload;
use feattle_core::persist::ValueHistory;
use feattle_core::FeattleDefinition;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The first version of the API. This is still unstable while this crate is in `0.x`
pub mod v1 {
    use super::*;

    #[derive(Debug, Clone, Serialize)]
    pub struct ListFeattlesResponse {
        pub definitions: Vec<FeattleDefinition>,
        pub last_reload: LastReload,
        pub reload_failed: bool,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct ShowFeattleResponse {
        pub definition: FeattleDefinition,
        pub history: ValueHistory,
        pub last_reload: LastReload,
        pub reload_failed: bool,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct EditFeattleRequest {
        pub key: String,
        pub value: Value,
        pub modified_by: String,
    }
}
