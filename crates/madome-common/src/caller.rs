use std::fmt;

use uuid::Uuid;

use crate::headers;

/// Caller role extracted from request headers/metadata.
///
/// Standalone enum without proto `Unspecified` — invalid states are
/// rejected at parse time, not carried through the system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallerRole {
    User,
    Admin,
    Owner,
}

impl fmt::Display for CallerRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallerRole::User => write!(f, "user"),
            CallerRole::Admin => write!(f, "admin"),
            CallerRole::Owner => write!(f, "owner"),
        }
    }
}

impl std::str::FromStr for CallerRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("user") {
            Ok(CallerRole::User)
        } else if s.eq_ignore_ascii_case("admin") {
            Ok(CallerRole::Admin)
        } else if s.eq_ignore_ascii_case("owner") {
            Ok(CallerRole::Owner)
        } else {
            Err(format!("invalid caller role: {s}"))
        }
    }
}

/// Validated caller identity extracted from HTTP headers or gRPC metadata.
///
/// Shared across Gateway and all gRPC services. Gateway extracts from
/// HTTP headers and injects into gRPC metadata; services extract from
/// gRPC metadata and convert `CallerRole` to their domain role type.
#[derive(Clone, Debug)]
pub struct CallerIdentity {
    pub caller_id: Uuid,
    pub caller_role: CallerRole,
}

impl CallerIdentity {
    /// Extracts mandatory caller identity from gRPC request metadata.
    pub fn from_metadata<T>(request: &tonic::Request<T>) -> Result<Self, tonic::Status> {
        let metadata = request.metadata();

        let caller_id = metadata
            .get(headers::X_CALLER_ID)
            .ok_or_else(|| tonic::Status::unauthenticated("missing x-caller-id"))?
            .to_str()
            .map_err(|_| tonic::Status::invalid_argument("invalid x-caller-id header"))?
            .parse::<Uuid>()
            .map_err(|_| tonic::Status::invalid_argument("invalid x-caller-id format"))?;

        let caller_role = metadata
            .get(headers::X_CALLER_ROLE)
            .ok_or_else(|| tonic::Status::unauthenticated("missing x-caller-role"))?
            .to_str()
            .map_err(|_| tonic::Status::invalid_argument("invalid x-caller-role header"))?
            .parse::<CallerRole>()
            .map_err(|_| tonic::Status::invalid_argument("invalid x-caller-role value"))?;

        Ok(CallerIdentity {
            caller_id,
            caller_role,
        })
    }

    /// Injects caller identity into tonic gRPC request metadata.
    pub fn inject_into<T>(&self, request: &mut tonic::Request<T>) {
        let metadata = request.metadata_mut();
        metadata.insert(
            headers::X_CALLER_ID,
            self.caller_id
                .to_string()
                .parse()
                .expect("UUID is valid ASCII"),
        );
        metadata.insert(
            headers::X_CALLER_ROLE,
            self.caller_role
                .to_string()
                .parse()
                .expect("role is valid ASCII"),
        );
    }
}
