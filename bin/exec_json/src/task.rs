//! Task description definition.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TaskDescription {
    /// Task level switch.
    /// If set to `false`, this task will stop executing.
    /// 
    /// This is designed as an emergency kill switch.
    #[serde(alias = "activate")]
    #[serde(alias = "activated")]
    #[serde(alias = "enable")]
    #[serde(alias = "enabled")]
    #[serde(alias = "on")]
    #[serde(default)]
    pub active: bool,

    /// Task query expression.
    // #[serde(alias = "query")]
    // #[serde(alias = "expression")]
    // pub expr: String,

    /// Eager mode flag.
    /// 
    /// If the task is in "eager mode", the output page will be actively written,
    /// even if the query failed. This may result in mass removal or addition of text.
    /// By default, when a task fails, only the header part will be updated.
    /// 
    /// This field can be omitted from the JSON configuration, and is defaulted to `false`.
    #[serde(default)]
    pub eager: bool,

    /// Task local configuration. Fields in these fields can override global configurations.
    #[serde(default)]
    pub timeout: Option<u64>,

    #[serde(alias = "limit")]
    #[serde(alias = "querylimit")]
    #[serde(default)]
    pub query_limit: Option<intorinf::IntOrInf>,

    /// Output specification of the task.
    /// This field can be omitted, and there will be no output at all.
    /// The `String` typed key is the title of the output page.
    #[serde(default = "collections::BTreeMap::new")]
    pub output: collections::BTreeMap<String, OutputFormat>,
}

/// Output format specification.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OutputFormat {
    /// Eager mode flag for this output page.
    /// This can be used to override the task-level eager mode flag.
    #[serde(default)]
    pub eager: Option<bool>,

    /// Things to write when the query fails.
    /// This field can be omitted, in this case, output is an empty string.
    #[serde(alias = "failure")]
    #[serde(alias = "error")]
    #[serde(default)]
    pub fail: String,

    /// Things to write when the query has zero results.
    /// This field can be omitted, in this case, output is an empty string.
    #[serde(alias = "zero")]
    #[serde(alias = "none")]
    #[serde(default)]
    pub empty: String,

    /// Things to write when the query succeeds.
    /// This field can be omitted, in this case, output is an empty string.
    #[serde(alias = "format")]
    #[serde(default)]
    pub success: OutputFormatSuccess,
}

/// Detailed output format when the query succeeds.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OutputFormatSuccess {
    /// Things to write before all output.
    /// This field can be omitted, in this case, output is an empty string.
    #[serde(alias = "start")]
    #[serde(alias = "begin")]
    #[serde(alias = "head")]
    #[serde(alias = "prepend")]
    #[serde(default)]
    pub before: String,

    /// Things to write for each item.
    /// This field can be omitted, in this case, output is an empty string.
    #[serde(default)]
    pub item: String,

    /// Things to write between two items.
    /// This field can be omitted, in this case, output is an empty string.
    #[serde(alias = "inside")]
    #[serde(default)]
    pub between: String,

    /// Things to write after all output.
    /// This field can be omitted, in this case, output is an empty string.
    #[serde(alias = "end")]
    #[serde(alias = "finish")]
    #[serde(alias = "tail")]
    #[serde(alias = "append")]
    #[serde(default)]
    pub after: String,
}
