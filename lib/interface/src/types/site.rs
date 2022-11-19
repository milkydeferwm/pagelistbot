//! On site types.
//! These types are both `Serialize` and `Deserialize`.

use std::collections;
#[cfg(feature = "use_serde")]
use serde::{Serialize, Deserialize};

use crate::types::ast::NumberOrInf;

/// Global configuration for Page List Bot host.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "use_serde",
    derive(Serialize, Deserialize),
)]
pub struct HostConfig {
    /// Global level host switch.
    /// If set to `false`, all tasks will stop executing.
    /// 
    /// This is designed as an emergency kill switch.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "activate"),
        serde(alias = "activated"),
        serde(alias = "enable"),
        serde(alias = "enabled"),
        serde(alias = "on"),
        serde(default),
    )]
    pub active: bool,

    /// The prefix index to look for tasks.
    /// Only pages with the given title prefix could be regarded as tasks.
    /// 
    /// Together with MediaWiki protection, this is a safety measure against abuse.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "taskdir"),
        serde(alias = "dir"),
        serde(alias = "prefix"),
    )]
    pub task_dir: String,

    /// The header template used in all output pages.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "resultheader"),
        serde(alias = "result_header"),
    )]
    pub header: String,

    /// If the output page falls in one of these namespaces,
    /// Page List Bot will not write to the page.
    /// 
    /// This is designed as a safety measure against abuse.
    /// Usually you don't want it to write to main namespace articles.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "denyns"),
        serde(default = "collections::BTreeSet::new"),
    )]
    pub deny_ns: collections::BTreeSet<i32>,

    /// The global default task configuration. Refer to the doc of `TaskConfig` for more information.
    /// This field is mandatory.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "default"),
    )]
    pub default_task_config: TaskConfig,
}

/// Task's configuration. This affects how the host executes the task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct TaskConfig {
    /// Timeout for the task.
    /// If timeout expires, the task will be killed,
    /// and a `timeout` error will be returned.
    /// 
    /// When used as a global configuration,
    /// it is applied to all tasks.
    /// It can be overridden by a task-local configuration.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "time"),
        serde(default),
    )]
    pub timeout: u64,

    /// Default query limit for the task.
    /// This is used in solvers.
    /// 
    /// When used as a global configuration,
    /// it is applied to all tasks.
    /// It can be overridden by a task-local configuration.
    /// Task local level configuration can again be overridden by expression inline `limit` modifiers.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "limit"),
        serde(alias = "querylimit"),
        serde(default),
    )]
    pub query_limit: NumberOrInf<usize>,
}

/// Task's configuration per task. This affects how the host executes the task.
/// It is manually made in sync with `TaskConfig`. If there is a way to eliminate this duplicate, let me know!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct OptionalTaskConfig {
    #[cfg_attr(feature = "use_serde",
        serde(default),
    )]
    pub timeout: Option<u64>,

    #[cfg_attr(feature = "use_serde",
        serde(alias = "limit"),
        serde(alias = "querylimit"),
        serde(default),
    )]
    pub query_limit: Option<NumberOrInf<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct TaskDescription {
    /// Task level switch.
    /// If set to `false`, this task will stop executing.
    /// 
    /// This is designed as an emergency kill switch.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "activate"),
        serde(alias = "activated"),
        serde(alias = "enable"),
        serde(alias = "enabled"),
        serde(default),
    )]
    pub active: bool,

    /// Optional description of the task.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "desc"),
        serde(default),
    )]
    pub description: String,

    /// Task query expression.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "query"),
        serde(alias = "expression"),
    )]
    pub expr: String,

    /// Task schedule.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "schedule"),
    )]
    pub cron: String,

    /// Eager mode flag.
    /// 
    /// If the task is in "eager mode", the output page will be actively written,
    /// even if the query failed. This may result in mass removal or addition of text.
    /// By default, when a task fails, only the header part will be updated.
    /// 
    /// This field can be omitted from the JSON configuration, and is defaulted to `false`.
    #[cfg_attr(feature = "use_serde",
        serde(default),
    )]
    pub eager: bool,

    /// Task local configuration. Fields in the struct can override global configurations.
    #[cfg_attr(feature = "use_serde",
        serde(default),
    )]
    pub config: Option<OptionalTaskConfig>,

    /// Output specification of the task.
    /// This field can be omitted, and there will be no output at all.
    /// The `String` typed key is the title of the output page.
    #[cfg_attr(feature = "use_serde",
        serde(default = "collections::BTreeMap::new"),
    )]
    pub output: collections::BTreeMap<String, OutputFormat>,
}

/// Output format specification.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct OutputFormat {
    /// Eager mode flag for this output page.
    /// This can be used to override the task-level eager mode flag.
    #[cfg_attr(feature = "use_serde",
        serde(default),
    )]
    pub eager: Option<bool>,

    /// Things to write when the query fails.
    /// This field can be omitted, in this case, output is an empty string.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "failure"),
        serde(alias = "error"),
        serde(default),
    )]
    pub fail: String,

    /// Things to write when the query has zero results.
    /// This field can be omitted, in this case, output is an empty string.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "zero"),
        serde(alias = "none"),
        serde(default),
    )]
    pub empty: String,

    /// Things to write when the query succeeds.
    /// This field can be omitted, in this case, output is an empty string.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "format"),
        serde(default),
    )]
    pub success: OutputFormatSuccess,
}

/// Detailed output format when the query succeeds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct OutputFormatSuccess {
    /// Things to write before all output.
    /// This field can be omitted, in this case, output is an empty string.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "start"),
        serde(alias = "begin"),
        serde(alias = "head"),
        serde(alias = "prepend"),
        serde(default),
    )]
    pub before: String,

    /// Things to write for each item.
    /// This field can be omitted, in this case, output is an empty string.
    #[cfg_attr(feature = "use_serde",
        serde(default),
    )]
    pub item: String,

    /// Things to write between two items.
    /// This field can be omitted, in this case, output is an empty string.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "inside"),
        serde(default),
    )]
    pub between: String,

    /// Things to write after all output.
    /// This field can be omitted, in this case, output is an empty string.
    #[cfg_attr(feature = "use_serde",
        serde(alias = "end"),
        serde(alias = "finish"),
        serde(alias = "tail"),
        serde(alias = "append"),
        serde(default),
    )]
    pub after: String,
}
