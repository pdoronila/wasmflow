//! Component metadata cache system
//!
//! This module provides caching for WASM component metadata to improve startup performance.
//! Component metadata (ComponentSpec) is extracted once and cached to disk. MD5 checksums
//! are used to detect when component files change and invalidate the cache.
//!
//! Cache location: `components/bin/.cache/`
//! Cache files:
//! - `<component-name>.json` - Cached ComponentSpec
//! - `<component-name>.md5` - MD5 checksum of the .wasm file
//! - `cache_version.txt` - Cache format version

use crate::graph::ComponentSpec;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Current cache format version
const CACHE_VERSION: &str = "1.0";

/// Cached component specification with metadata
#[derive(Debug, Serialize, Deserialize)]
struct CachedComponentSpec {
    /// Cache format version
    version: String,
    /// MD5 checksum of the WASM file
    checksum: String,
    /// Timestamp when cached
    cached_at: String,
    /// The cached component specification
    component_spec: ComponentSpec,
}

/// Component metadata cache manager
///
/// Manages a disk-based cache of component metadata to avoid expensive
/// WASM parsing and metadata extraction on every application startup.
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use wasmflow::runtime::ComponentCache;
///
/// let cache = ComponentCache::new(PathBuf::from("components/bin/.cache"))?;
///
/// // Try to load from cache
/// let wasm_path = PathBuf::from("components/bin/echo.wasm");
/// if let Some(spec) = cache.get_cached_spec(&wasm_path)? {
///     println!("Loaded {} from cache", spec.name);
/// } else {
///     // Extract metadata and cache it
///     let spec = extract_metadata(&wasm_path)?;
///     cache.save_spec(&wasm_path, &spec)?;
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct ComponentCache {
    cache_dir: PathBuf,
    cache_version: String,
}

impl ComponentCache {
    /// Create a new component cache
    ///
    /// Creates the cache directory if it doesn't exist and initializes
    /// the cache version file.
    ///
    /// # Arguments
    ///
    /// * `cache_dir` - Path to the cache directory (e.g., `components/bin/.cache`)
    ///
    /// # Returns
    ///
    /// Returns the cache manager or an error if directory creation fails
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        log::debug!("Initializing component cache at: {}", cache_dir.display());

        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;
            log::info!("Created cache directory: {}", cache_dir.display());
        }

        let cache = Self {
            cache_dir,
            cache_version: CACHE_VERSION.to_string(),
        };

        // Initialize or validate cache version
        cache.init_cache_version()?;

        Ok(cache)
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get a cached component spec if available and valid
    ///
    /// # Arguments
    ///
    /// * `wasm_path` - Path to the WASM component file
    ///
    /// # Returns
    ///
    /// - `Ok(Some(spec))` if cache hit and valid
    /// - `Ok(None)` if cache miss or invalid
    /// - `Err(e)` if I/O error reading cache
    pub fn get_cached_spec(&self, wasm_path: &Path) -> Result<Option<ComponentSpec>> {
        let component_name = self.get_component_name(wasm_path)?;
        let cache_path = self.cache_dir.join(format!("{}.json", component_name));

        // Check if cache file exists
        if !cache_path.exists() {
            log::debug!("Cache miss (no file): {}", component_name);
            return Ok(None);
        }

        // Validate cache against current WASM file
        if !self.is_cache_valid(wasm_path, &cache_path)? {
            log::info!("Cache invalid (checksum mismatch): {}", component_name);
            return Ok(None);
        }

        // Load cached spec
        match self.load_cached_spec(&cache_path) {
            Ok(cached) => {
                // Validate cache format version
                if cached.version != self.cache_version {
                    log::warn!(
                        "Cache version mismatch for {}: expected {}, got {}",
                        component_name,
                        self.cache_version,
                        cached.version
                    );
                    return Ok(None);
                }

                log::info!("Cache hit: {} (cached at {})", component_name, cached.cached_at);
                Ok(Some(cached.component_spec))
            }
            Err(e) => {
                log::warn!("Failed to load cache for {}: {}", component_name, e);
                // Delete corrupted cache file
                let _ = fs::remove_file(&cache_path);
                Ok(None)
            }
        }
    }

    /// Save a component spec to cache
    ///
    /// # Arguments
    ///
    /// * `wasm_path` - Path to the WASM component file
    /// * `spec` - Component specification to cache
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success or an error if caching fails
    pub fn save_spec(&self, wasm_path: &Path, spec: &ComponentSpec) -> Result<()> {
        let component_name = self.get_component_name(wasm_path)?;
        let cache_path = self.cache_dir.join(format!("{}.json", component_name));
        let md5_path = self.cache_dir.join(format!("{}.md5", component_name));

        // Compute checksum
        let checksum = Self::compute_checksum(wasm_path)
            .with_context(|| format!("Failed to compute checksum for {}", wasm_path.display()))?;

        // Create cached spec wrapper
        let cached = CachedComponentSpec {
            version: self.cache_version.clone(),
            checksum: checksum.clone(),
            cached_at: Utc::now().to_rfc3339(),
            component_spec: spec.clone(),
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&cached)
            .context("Failed to serialize cached spec")?;

        // Write cache file
        fs::write(&cache_path, json)
            .with_context(|| format!("Failed to write cache file: {}", cache_path.display()))?;

        // Write MD5 file
        fs::write(&md5_path, checksum)
            .with_context(|| format!("Failed to write MD5 file: {}", md5_path.display()))?;

        log::debug!("Cached component: {} at {}", component_name, cache_path.display());
        Ok(())
    }

    /// Invalidate all cached entries
    ///
    /// Deletes all cache files (*.json and *.md5) but preserves the cache directory
    /// and version file.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success or an error if deletion fails
    pub fn invalidate_all(&self) -> Result<()> {
        log::info!("Invalidating all cache entries in: {}", self.cache_dir.display());

        let mut deleted_count = 0;
        let mut error_count = 0;

        // Read cache directory
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Skip the version file
                if path.file_name() == Some(std::ffi::OsStr::new("cache_version.txt")) {
                    continue;
                }

                // Delete cache files (.json and .md5)
                if let Some(ext) = path.extension() {
                    if ext == "json" || ext == "md5" {
                        match fs::remove_file(&path) {
                            Ok(()) => {
                                deleted_count += 1;
                                log::debug!("Deleted cache file: {}", path.display());
                            }
                            Err(e) => {
                                error_count += 1;
                                log::warn!("Failed to delete {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        log::info!(
            "Cache invalidation complete: {} files deleted, {} errors",
            deleted_count,
            error_count
        );

        if error_count > 0 {
            anyhow::bail!("Failed to delete {} cache files", error_count);
        }

        Ok(())
    }

    /// Get cache statistics
    ///
    /// Returns information about the cache including number of entries and disk usage
    pub fn get_statistics(&self) -> Result<CacheStatistics> {
        let mut json_count = 0;
        let mut total_size = 0u64;

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "json" {
                        json_count += 1;
                        if let Ok(metadata) = fs::metadata(&path) {
                            total_size += metadata.len();
                        }
                    }
                }
            }
        }

        Ok(CacheStatistics {
            cached_components: json_count,
            total_size_bytes: total_size,
            cache_dir: self.cache_dir.clone(),
        })
    }

    // Private helper methods

    /// Initialize or validate cache version file
    fn init_cache_version(&self) -> Result<()> {
        let version_path = self.cache_dir.join("cache_version.txt");

        if version_path.exists() {
            // Validate existing version
            let existing_version = fs::read_to_string(&version_path)
                .context("Failed to read cache version file")?;

            if existing_version.trim() != self.cache_version {
                log::warn!(
                    "Cache version mismatch: expected {}, found {}. Invalidating cache.",
                    self.cache_version,
                    existing_version.trim()
                );
                self.invalidate_all()?;
                // Write new version
                fs::write(&version_path, &self.cache_version)
                    .context("Failed to write cache version file")?;
            }
        } else {
            // Create new version file
            fs::write(&version_path, &self.cache_version)
                .context("Failed to write cache version file")?;
            log::info!("Created cache version file: {}", self.cache_version);
        }

        Ok(())
    }

    /// Extract component name from WASM file path
    fn get_component_name(&self, wasm_path: &Path) -> Result<String> {
        wasm_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Invalid WASM file path: {}", wasm_path.display()))
    }

    /// Compute MD5 checksum of a file
    ///
    /// # Arguments
    ///
    /// * `wasm_path` - Path to the WASM file
    ///
    /// # Returns
    ///
    /// Returns hex-encoded MD5 checksum or an error if file read fails
    pub fn compute_checksum(wasm_path: &Path) -> Result<String> {
        log::debug!("Computing MD5 checksum for: {}", wasm_path.display());

        let bytes = fs::read(wasm_path)
            .with_context(|| format!("Failed to read file: {}", wasm_path.display()))?;

        let digest = md5::compute(&bytes);
        let checksum = format!("{:x}", digest);

        log::debug!("Checksum for {}: {}", wasm_path.display(), checksum);
        Ok(checksum)
    }

    /// Check if cache is valid by comparing checksums
    ///
    /// # Arguments
    ///
    /// * `wasm_path` - Path to the WASM component file
    /// * `cache_path` - Path to the cache JSON file
    ///
    /// # Returns
    ///
    /// Returns true if cache is valid (checksums match), false otherwise
    fn is_cache_valid(&self, wasm_path: &Path, cache_path: &Path) -> Result<bool> {
        let component_name = self.get_component_name(wasm_path)?;
        let md5_path = self.cache_dir.join(format!("{}.md5", component_name));

        // Check if MD5 file exists
        if !md5_path.exists() {
            log::debug!("MD5 file missing for {}", component_name);
            return Ok(false);
        }

        // Read cached checksum
        let cached_checksum = fs::read_to_string(&md5_path)
            .with_context(|| format!("Failed to read MD5 file: {}", md5_path.display()))?;

        // Compute current checksum
        let current_checksum = Self::compute_checksum(wasm_path)?;

        // Compare checksums
        let valid = cached_checksum.trim() == current_checksum.trim();

        if !valid {
            log::debug!(
                "Checksum mismatch for {}: cached={}, current={}",
                component_name,
                cached_checksum.trim(),
                current_checksum.trim()
            );
        }

        Ok(valid)
    }

    /// Load cached spec from JSON file
    fn load_cached_spec(&self, cache_path: &Path) -> Result<CachedComponentSpec> {
        let json = fs::read_to_string(cache_path)
            .with_context(|| format!("Failed to read cache file: {}", cache_path.display()))?;

        let cached: CachedComponentSpec = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse cache file: {}", cache_path.display()))?;

        Ok(cached)
    }
}

/// Cache statistics information
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    /// Number of cached components
    pub cached_components: usize,
    /// Total disk space used by cache (bytes)
    pub total_size_bytes: u64,
    /// Cache directory path
    pub cache_dir: PathBuf,
}

impl CacheStatistics {
    /// Get total size in megabytes
    pub fn total_size_mb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_wasm_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(format!("{}.wasm", name));
        fs::write(&path, content).unwrap();
        path
    }

    fn create_test_spec(name: &str) -> ComponentSpec {
        ComponentSpec {
            id: format!("test:{}", name),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: "Test component".to_string(),
            author: "Test".to_string(),
            category: Some("Test".to_string()),
            inputs: vec![],
            outputs: vec![],
            capabilities: vec![],
        }
    }

    #[test]
    fn test_cache_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".cache");

        let cache = ComponentCache::new(cache_dir.clone()).unwrap();

        assert!(cache_dir.exists());
        assert!(cache_dir.join("cache_version.txt").exists());
    }

    #[test]
    fn test_compute_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let wasm_path = create_test_wasm_file(temp_dir.path(), "test", b"hello world");

        let checksum1 = ComponentCache::compute_checksum(&wasm_path).unwrap();
        let checksum2 = ComponentCache::compute_checksum(&wasm_path).unwrap();

        // Same file should produce same checksum
        assert_eq!(checksum1, checksum2);

        // Modify file
        fs::write(&wasm_path, b"hello world!").unwrap();
        let checksum3 = ComponentCache::compute_checksum(&wasm_path).unwrap();

        // Different content should produce different checksum
        assert_ne!(checksum1, checksum3);
    }

    #[test]
    fn test_cache_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".cache");
        let cache = ComponentCache::new(cache_dir).unwrap();

        let wasm_path = create_test_wasm_file(temp_dir.path(), "echo", b"test wasm content");
        let spec = create_test_spec("echo");

        // Save to cache
        cache.save_spec(&wasm_path, &spec).unwrap();

        // Load from cache
        let loaded = cache.get_cached_spec(&wasm_path).unwrap();
        assert!(loaded.is_some());

        let loaded_spec = loaded.unwrap();
        assert_eq!(loaded_spec.id, spec.id);
        assert_eq!(loaded_spec.name, spec.name);
    }

    #[test]
    fn test_cache_invalidation_on_file_change() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".cache");
        let cache = ComponentCache::new(cache_dir).unwrap();

        let wasm_path = create_test_wasm_file(temp_dir.path(), "echo", b"version 1");
        let spec = create_test_spec("echo");

        // Cache the spec
        cache.save_spec(&wasm_path, &spec).unwrap();

        // Verify cache hit
        let loaded = cache.get_cached_spec(&wasm_path).unwrap();
        assert!(loaded.is_some());

        // Modify the WASM file
        fs::write(&wasm_path, b"version 2").unwrap();

        // Cache should be invalid now
        let loaded = cache.get_cached_spec(&wasm_path).unwrap();
        assert!(loaded.is_none(), "Cache should be invalid after file modification");
    }

    #[test]
    fn test_cache_miss_for_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".cache");
        let cache = ComponentCache::new(cache_dir).unwrap();

        let wasm_path = temp_dir.path().join("nonexistent.wasm");

        // Should return None for missing cache
        let result = cache.get_cached_spec(&wasm_path);
        assert!(result.is_err() || result.unwrap().is_none());
    }

    #[test]
    fn test_invalidate_all() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".cache");
        let cache = ComponentCache::new(cache_dir.clone()).unwrap();

        // Cache multiple components
        for i in 0..5 {
            let name = format!("comp{}", i);
            let wasm_path = create_test_wasm_file(temp_dir.path(), &name, b"test");
            let spec = create_test_spec(&name);
            cache.save_spec(&wasm_path, &spec).unwrap();
        }

        // Verify files exist
        assert!(cache_dir.join("comp0.json").exists());
        assert!(cache_dir.join("comp0.md5").exists());

        // Invalidate all
        cache.invalidate_all().unwrap();

        // Verify cache files deleted but directory and version file remain
        assert!(!cache_dir.join("comp0.json").exists());
        assert!(!cache_dir.join("comp0.md5").exists());
        assert!(cache_dir.exists());
        assert!(cache_dir.join("cache_version.txt").exists());
    }

    #[test]
    fn test_cache_statistics() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".cache");
        let cache = ComponentCache::new(cache_dir).unwrap();

        // Initially empty
        let stats = cache.get_statistics().unwrap();
        assert_eq!(stats.cached_components, 0);

        // Cache some components
        for i in 0..3 {
            let name = format!("comp{}", i);
            let wasm_path = create_test_wasm_file(temp_dir.path(), &name, b"test");
            let spec = create_test_spec(&name);
            cache.save_spec(&wasm_path, &spec).unwrap();
        }

        let stats = cache.get_statistics().unwrap();
        assert_eq!(stats.cached_components, 3);
        assert!(stats.total_size_bytes > 0);
    }
}
