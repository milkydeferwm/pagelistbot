//! Commands that are sent from frontend to backend daemon.

use serde::{Serialize, Deserialize};

/// `NewHost` command requires the backend to initialize a new `Host` instance.
/// The new instance will be inserted into the backend's host map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewHost {
    /// The name assigned to the host. This will be used as the host map's key.
    pub name: String,
    /// The API endpoint for the `Host` to work on.
    pub api_endpoint: String,
    /// For API: the login username.
    pub username: String,
    /// For API: the login password.
    pub password: String,
    /// For API: prefer editing with bot flag when possible.
    /// This field can be omitted and in such case, this is assumed to be `false`.
    #[serde(default)]
    pub edit_prefer_bot: bool,
    /// For direct database access: the database name. Not in every case the bot can access the database.
    /// This field can be omitted and in such case, this is assumed to be `None`. Solving queries will then use pure API.
    #[serde(default)]
    pub db_name: Option<String>,
}
