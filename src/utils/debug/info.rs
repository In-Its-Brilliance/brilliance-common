use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{fmt, time::Duration};

use crate::utils::{human_number, humanize_key};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DebugValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Duration {
        ms: f64,
    },
    #[serde(skip)]
    Colored {
        value: Box<DebugValue>,
        color: &'static str,
    },
}

impl DebugValue {
    pub fn with_color(self, color: &'static str) -> Self {
        Self::Colored {
            value: Box::new(self),
            color,
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Colored { color, .. } => color,
            _ => "&f",
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(v) => Some(*v as f64),
            Self::Float(v) => Some(*v),
            Self::Duration { ms } => Some(*ms),
            Self::Colored { value, .. } => value.as_f64(),
            _ => None,
        }
    }
}

impl fmt::Display for DebugValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{}", human_number(*v)),
            Self::Float(v) => write!(f, "{:.1}", v),
            Self::Bool(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
            Self::Duration { ms } => {
                if *ms >= 1000.0 {
                    write!(f, "{:>6.1} s", ms / 1000.0)
                } else if *ms >= 1.0 {
                    write!(f, "{:>6.1}ms", ms)
                } else {
                    write!(f, "{:>6.1}us", ms * 1000.0)
                }
            }
            Self::Colored { value, .. } => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DebugInfo {
    debug_info: IndexMap<String, DebugValue>,
}

impl DebugInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(mut self, key: impl Into<String>, value: impl Into<DebugValue>) -> Self {
        self.debug_info.insert(key.into(), value.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&DebugValue> {
        self.debug_info.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &DebugValue)> {
        self.debug_info.iter()
    }

    pub fn get_console_print(&self, cols: usize, title_color: &'static str) -> String {
        let parts: Vec<(String, usize)> = self
            .iter()
            .map(|(title, value)| {
                let title_str = humanize_key(title);
                let value_str = value.to_string();
                let color = value.color();
                let visible_len = title_str.len() + value_str.len() + 1;
                (
                    format!(
                        "{title_color}{title} {color}{value}&r",
                        title = title_str,
                        color = color,
                        value = value_str
                    ),
                    visible_len,
                )
            })
            .collect();

        let col_width: usize = 22;
        let cols = cols.max(1);
        let mut lines = Vec::new();
        
        for chunk in parts.chunks(cols) {
            let line: Vec<String> = chunk
                .iter()
                .enumerate()
                .map(|(i, (pair, len))| {
                    if i < chunk.len() - 1 {
                        let padding = col_width.saturating_sub(*len);
                        format!("{}{}", pair, " ".repeat(padding))
                    } else {
                        pair.clone()
                    }
                })
                .collect();
            lines.push(line.join(" &f|&r "));
        }
        
        lines.join("\n")
    }
}

// Конверсии для DebugValue
impl From<i64> for DebugValue {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<i32> for DebugValue {
    fn from(v: i32) -> Self {
        Self::Int(v as i64)
    }
}

impl From<usize> for DebugValue {
    fn from(v: usize) -> Self {
        Self::Int(v as i64)
    }
}

impl From<f64> for DebugValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<f32> for DebugValue {
    fn from(v: f32) -> Self {
        Self::Float(v as f64)
    }
}

impl From<bool> for DebugValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<String> for DebugValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for DebugValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl From<Duration> for DebugValue {
    fn from(v: Duration) -> Self {
        Self::Duration {
            ms: v.as_secs_f64() * 1000.0,
        }
    }
}
