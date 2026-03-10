use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NumberLit(pub f32);
impl<T: Into<f32>> From<T> for NumberLit {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}
impl fmt::Display for NumberLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringLit(pub String);
impl<T: AsRef<str>> From<T> for StringLit {
    fn from(value: T) -> Self {
        Self(value.as_ref().to_string())
    }
}
impl fmt::Display for StringLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BooleanLit(pub bool);
impl<T: Into<bool>> From<T> for BooleanLit {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}
impl fmt::Display for BooleanLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListLit(pub Vec<Value>);
impl<T: Into<Vec<Value>>> From<T> for ListLit {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}
impl fmt::Display for ListLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0.iter().fold(
            String::new(), |acc, item| {
                if acc.is_empty() {
                    item.to_string()
                } else {
                    format!("{acc}, {item}")
                }
            }
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Number(NumberLit),
    String(StringLit),
    Boolean(BooleanLit),
    List(ListLit),
    Nil
}
impl Value {
    pub fn as_f32(&self) -> f32 {
        match self {
            Self::Number(n) => n.0,
            Self::String(n) => n.0.parse().unwrap_or(0.0),
            Self::Boolean(n) => n.0 as i32 as f32,
            _ => 0.0
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Number(n) => n.0 != 0.0,
            Self::String(n) => {
                let n = &n.0;
                if let Ok(n) = n.parse() { return n }
                else if let Ok(n) = n.parse::<i32>() { return n != 0 }
                else if let Ok(n) = n.parse::<f32>() { return n != 0.0 }
                else { false }
            },
            Self::Boolean(b) => b.0,
            _ => false,
        }
    }

    pub fn change_by(&mut self, other: &Self) {
        match (self, other) {
            (Self::Number(n), Self::Number(v)) => n.0 += v.0,
            (Self::Number(n), Self::String(v)) => n.0 += v.0.parse().unwrap_or(0.0),
            (Self::Number(n), Self::Boolean(v)) => n.0 += v.0 as i32 as f32,
            (Self::String(n), Self::Number(v)) => n.0 += &v.0.to_string(),
            (Self::String(n), Self::String(v)) => n.0 += &v.0,
            (Self::String(n), Self::Boolean(v)) => n.0 += &v.0.to_string(),
            (Self::List(n), Self::List(v)) => n.0.extend(v.0.clone()),
            (Self::List(n), _) => n.0.push(other.clone()),
            _ => (),
        }
    }
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => n.fmt(f),
            Self::String(n) => n.fmt(f),
            Self::Boolean(n) => n.fmt(f),
            Self::List(n) => n.fmt(f),
            Self::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Expression {
    Value(Value),
    Variable(String),
    GlobalVar(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum Block {
    ChangeXBy(Expression),
    ChangeYBy(Expression),
    SetPositionTo { x: Expression, y: Expression },

    If {
        condition: Expression,
        then: Vec<Block>,
    },
    IfElse {
        condition: Expression,
        then: Vec<Block>,
        #[serde(rename = "else")]
        else_: Vec<Block>,
    },

    ChangeVarBy {
        name: String,
        value: Expression,
    },
    SetVarTo {
        name: String,
        value: Expression,
    },

    Show,
    Hide,

    StopThisScript,
    StopOtherScripts,
    StopAllScriptsInThisEntity,
    StopAllScripts,

    WaitSeconds(Expression),
}