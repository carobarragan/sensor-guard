//! src/enums/role_enum.rs — User role enum with Display, FromStr, and SQLx support.
//!

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// User roles for RBAC authorization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Operator,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Operator => write!(f, "operator"),
        }
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(Role::Admin),
            "operator" => Ok(Role::Operator),
            other => Err(format!("Unknown role: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_display_admin_should_return_admin() {
        assert_eq!(Role::Admin.to_string(), "admin");
    }

    #[test]
    fn when_display_operator_should_return_operator() {
        assert_eq!(Role::Operator.to_string(), "operator");
    }

    #[test]
    fn when_parsing_admin_should_succeed() {
        assert_eq!(Role::from_str("admin").unwrap(), Role::Admin);
    }

    #[test]
    fn when_parsing_operator_should_succeed() {
        assert_eq!(Role::from_str("operator").unwrap(), Role::Operator);
    }

    #[test]
    fn when_parsing_unknown_should_fail() {
        assert!(Role::from_str("superadmin").is_err());
    }

    #[test]
    fn when_serializing_admin_should_be_lowercase() {
        let json = serde_json::to_string(&Role::Admin).unwrap();
        assert_eq!(json, "\"admin\"");
    }

    #[test]
    fn when_deserializing_operator_should_succeed() {
        let role: Role = serde_json::from_str("\"operator\"").unwrap();
        assert_eq!(role, Role::Operator);
    }
}

// End of File
