//! Capability-based security system for node execution
//!
//! Implements the capability-based security model defined in the constitution.
//! Nodes must declare required capabilities, and users must grant them explicitly.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// System capabilities that nodes can request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Read files from the filesystem
    FileRead,
    /// Write files to the filesystem
    FileWrite,
    /// Execute external processes
    ProcessSpawn,
    /// Make network requests (HTTP/HTTPS)
    NetworkHttp,
    /// Access raw TCP/UDP sockets
    NetworkSocket,
    /// Access environment variables
    EnvAccess,
    /// Access system time and clocks
    TimeAccess,
    /// Access cryptographic random number generation
    CryptoRandom,
}

impl Capability {
    /// Get a human-readable description of this capability
    pub fn description(&self) -> &'static str {
        match self {
            Capability::FileRead => "Read files from the filesystem",
            Capability::FileWrite => "Write files to the filesystem",
            Capability::ProcessSpawn => "Execute external programs",
            Capability::NetworkHttp => "Make HTTP/HTTPS network requests",
            Capability::NetworkSocket => "Access raw network sockets (TCP/UDP)",
            Capability::EnvAccess => "Read environment variables",
            Capability::TimeAccess => "Access system time and clocks",
            Capability::CryptoRandom => "Generate cryptographic random numbers",
        }
    }

    /// Get the risk level of this capability
    pub fn risk_level(&self) -> RiskLevel {
        match self {
            Capability::FileWrite | Capability::ProcessSpawn | Capability::NetworkSocket => {
                RiskLevel::High
            }
            Capability::FileRead | Capability::NetworkHttp | Capability::EnvAccess => {
                RiskLevel::Medium
            }
            Capability::TimeAccess | Capability::CryptoRandom => RiskLevel::Low,
        }
    }
}

/// Risk level classification for capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Set of capabilities granted to a node or component
/// Matches the data model specification with path/host restrictions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CapabilitySet {
    /// No system access (pure computation)
    #[default]
    None,
    /// Read-only access to specific directories
    FileRead { paths: Vec<PathBuf> },
    /// Write access to specific directories
    FileWrite { paths: Vec<PathBuf> },
    /// Combined read/write access
    FileReadWrite { paths: Vec<PathBuf> },
    /// HTTP access to allowlisted domains
    Network { allowed_hosts: Vec<String> },
    /// Unrestricted access (requires explicit warning)
    Full,
}

impl CapabilitySet {
    /// Create an empty capability set
    pub fn none() -> Self {
        CapabilitySet::None
    }

    /// Create file read capability set
    pub fn file_read(paths: Vec<PathBuf>) -> Self {
        CapabilitySet::FileRead { paths }
    }

    /// Create file write capability set
    pub fn file_write(paths: Vec<PathBuf>) -> Self {
        CapabilitySet::FileWrite { paths }
    }

    /// Create file read/write capability set
    pub fn file_read_write(paths: Vec<PathBuf>) -> Self {
        CapabilitySet::FileReadWrite { paths }
    }

    /// Create network capability set
    pub fn network(allowed_hosts: Vec<String>) -> Self {
        CapabilitySet::Network { allowed_hosts }
    }

    /// Create full capability set (dangerous)
    pub fn full() -> Self {
        CapabilitySet::Full
    }

    /// Check if a specific capability is granted
    pub fn has(&self, capability: Capability) -> bool {
        match (self, capability) {
            (CapabilitySet::None, _) => false,
            (CapabilitySet::FileRead { .. }, Capability::FileRead) => true,
            (CapabilitySet::FileWrite { .. }, Capability::FileWrite) => true,
            (CapabilitySet::FileReadWrite { .. }, Capability::FileRead | Capability::FileWrite) => true,
            (CapabilitySet::Network { .. }, Capability::NetworkHttp) => true,
            (CapabilitySet::Full, _) => true,
            _ => false,
        }
    }

    /// Get the highest risk level among granted capabilities
    pub fn max_risk_level(&self) -> Option<RiskLevel> {
        match self {
            CapabilitySet::None => None,
            CapabilitySet::FileRead { .. } => Some(RiskLevel::Medium),
            CapabilitySet::FileWrite { .. } => Some(RiskLevel::High),
            CapabilitySet::FileReadWrite { .. } => Some(RiskLevel::High),
            CapabilitySet::Network { .. } => Some(RiskLevel::Medium),
            CapabilitySet::Full => Some(RiskLevel::High),
        }
    }

    /// Get a human-readable description of the capability set
    pub fn description(&self) -> String {
        match self {
            CapabilitySet::None => "No system access (pure computation)".to_string(),
            CapabilitySet::FileRead { paths } => {
                format!("Read files from: {}", Self::format_paths(paths))
            }
            CapabilitySet::FileWrite { paths } => {
                format!("Write files to: {}", Self::format_paths(paths))
            }
            CapabilitySet::FileReadWrite { paths } => {
                format!("Read/write files in: {}", Self::format_paths(paths))
            }
            CapabilitySet::Network { allowed_hosts } => {
                format!("Network access to: {}", allowed_hosts.join(", "))
            }
            CapabilitySet::Full => "Full system access (all capabilities)".to_string(),
        }
    }

    fn format_paths(paths: &[PathBuf]) -> String {
        if paths.is_empty() {
            "none".to_string()
        } else {
            paths.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}

/// User-approved permission for a node to access system resources
/// T066: Implement CapabilityGrant struct
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityGrant {
    /// Node receiving the grant (UUID)
    pub node_id: Uuid,
    /// Capability set defining what's allowed
    pub capability_set: CapabilitySet,
    /// Timestamp of user approval (ISO 8601 format)
    pub granted_at: String,
    /// Specific restrictions (file paths, network hosts)
    pub scope: String,
}

impl CapabilityGrant {
    /// Create a new capability grant
    pub fn new(node_id: Uuid, capability_set: CapabilitySet) -> Self {
        let scope = capability_set.description();
        Self {
            node_id,
            capability_set,
            granted_at: chrono::Utc::now().to_rfc3339(),
            scope,
        }
    }

    /// Check if this grant satisfies the required capabilities
    pub fn satisfies(&self, required: &CapabilitySet) -> bool {
        match (&self.capability_set, required) {
            (_, CapabilitySet::None) => true,
            (CapabilitySet::Full, _) => true,
            (CapabilitySet::None, _) => false,
            (granted, req) => {
                // Both must be the same variant type
                std::mem::discriminant(granted) == std::mem::discriminant(req)
            }
        }
    }

    /// T067-T070: Convert CapabilitySet to WASI context builder
    /// This will be used by the runtime to configure the WASM sandbox
    /// Note: Currently implemented in wasm_host.rs configure_wasi() instead
    #[allow(dead_code)]
    pub fn to_wasi_ctx(&self) -> Result<wasmtime_wasi::WasiCtx, anyhow::Error> {
        use wasmtime_wasi::WasiCtxBuilder;

        let mut builder = WasiCtxBuilder::new();

        match &self.capability_set {
            CapabilitySet::None => {
                // Empty context - no system access
            }
            CapabilitySet::FileRead { paths } |
            CapabilitySet::FileWrite { paths } |
            CapabilitySet::FileReadWrite { paths } => {
                // T068-T069: File capability enforcement
                use wasmtime_wasi::{DirPerms, FilePerms};

                for path in paths {
                    if !path.is_absolute() {
                        anyhow::bail!("Path must be absolute: {:?}", path);
                    }
                    let path_str = path.to_str()
                        .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in path"))?;
                    builder.preopened_dir(
                        path.clone(),
                        path_str,
                        DirPerms::all(),
                        FilePerms::all(),
                    )?;
                }
            }
            CapabilitySet::Network { allowed_hosts: _ } => {
                // T070: Network capability enforcement
                builder.inherit_network();
            }
            CapabilitySet::Full => {
                // Full access - inherit everything
                builder
                    .inherit_stdio()
                    .inherit_env()
                    .inherit_network();
            }
        }

        Ok(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_capability_description() {
        assert_eq!(
            Capability::FileRead.description(),
            "Read files from the filesystem"
        );
        assert_eq!(
            Capability::NetworkHttp.description(),
            "Make HTTP/HTTPS network requests"
        );
    }

    #[test]
    fn test_risk_levels() {
        assert_eq!(Capability::FileWrite.risk_level(), RiskLevel::High);
        assert_eq!(Capability::FileRead.risk_level(), RiskLevel::Medium);
        assert_eq!(Capability::TimeAccess.risk_level(), RiskLevel::Low);
    }

    #[test]
    fn test_capability_set_has() {
        let none = CapabilitySet::none();
        assert!(!none.has(Capability::FileRead));

        let file_read = CapabilitySet::file_read(vec![PathBuf::from("/tmp")]);
        assert!(file_read.has(Capability::FileRead));
        assert!(!file_read.has(Capability::FileWrite));

        let file_read_write = CapabilitySet::file_read_write(vec![PathBuf::from("/tmp")]);
        assert!(file_read_write.has(Capability::FileRead));
        assert!(file_read_write.has(Capability::FileWrite));

        let full = CapabilitySet::full();
        assert!(full.has(Capability::FileRead));
        assert!(full.has(Capability::NetworkSocket));
    }

    #[test]
    fn test_max_risk_level() {
        let none = CapabilitySet::none();
        assert_eq!(none.max_risk_level(), None);

        let file_read = CapabilitySet::file_read(vec![PathBuf::from("/tmp")]);
        assert_eq!(file_read.max_risk_level(), Some(RiskLevel::Medium));

        let file_write = CapabilitySet::file_write(vec![PathBuf::from("/tmp")]);
        assert_eq!(file_write.max_risk_level(), Some(RiskLevel::High));

        let full = CapabilitySet::full();
        assert_eq!(full.max_risk_level(), Some(RiskLevel::High));
    }

    #[test]
    fn test_capability_set_description() {
        let none = CapabilitySet::none();
        assert_eq!(none.description(), "No system access (pure computation)");

        let file_read = CapabilitySet::file_read(vec![PathBuf::from("/tmp"), PathBuf::from("/data")]);
        assert!(file_read.description().contains("/tmp"));
        assert!(file_read.description().contains("/data"));

        let network = CapabilitySet::network(vec!["example.com".to_string()]);
        assert!(network.description().contains("example.com"));
    }

    #[test]
    fn test_capability_grant() {
        use uuid::Uuid;
        let node_id = Uuid::new_v4();
        let capability_set = CapabilitySet::file_read(vec![PathBuf::from("/data")]);

        let grant = CapabilityGrant::new(node_id, capability_set.clone());

        assert_eq!(grant.node_id, node_id);
        assert_eq!(grant.capability_set, capability_set);
        assert!(!grant.granted_at.is_empty());
        assert!(grant.scope.contains("/data"));
    }

    #[test]
    fn test_capability_grant_satisfies() {
        use uuid::Uuid;
        let node_id = Uuid::new_v4();

        let grant = CapabilityGrant::new(
            node_id,
            CapabilitySet::file_read(vec![PathBuf::from("/data")])
        );

        assert!(grant.satisfies(&CapabilitySet::None));
        assert!(grant.satisfies(&CapabilitySet::file_read(vec![PathBuf::from("/data")])));
        assert!(!grant.satisfies(&CapabilitySet::file_write(vec![PathBuf::from("/data")])));

        let full_grant = CapabilityGrant::new(node_id, CapabilitySet::full());
        assert!(full_grant.satisfies(&CapabilitySet::file_read(vec![PathBuf::from("/any")])));
    }
}
