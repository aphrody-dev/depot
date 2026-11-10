use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Binary header framing for Blake3 artifact caches and manifests using Google's zerocopy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct ArtifactBinaryHeader {
    pub magic: [u8; 8],        // b"DEPOTART"
    pub version: u32,          // 1
    pub entry_count: u32,
    pub blake3_digest: [u8; 32],
    pub total_bytes: u64,
}

impl ArtifactBinaryHeader {
    pub const MAGIC: [u8; 8] = *b"DEPOTART";
    pub const VERSION: u32 = 1;

    pub fn new(entry_count: u32, blake3_digest: [u8; 32], total_bytes: u64) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            entry_count,
            blake3_digest,
            total_bytes,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        zerocopy::IntoBytes::as_bytes(self)
    }

    pub fn parse_from_prefix(bytes: &[u8]) -> Option<(Self, &[u8])> {
        Self::read_from_prefix(bytes).ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactEntry {
    pub name: String,
    pub path: PathBuf,
    pub blake3_hash: String,
    pub size_bytes: u64,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub version: String,
    pub project_name: String,
    #[serde(default)]
    pub artifacts: Vec<ArtifactEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactReport {
    pub total_artifacts: usize,
    pub total_bytes: u64,
    pub verified_ok: usize,
    pub mismatched: Vec<String>,
    pub missing: Vec<String>,
}

pub struct Artifact;

impl Artifact {
    pub fn compute_blake3(path: &Path) -> Result<(String, u64), String> {
        let file = File::open(path).map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
        let metadata = file.metadata().map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 65536];

        loop {
            let n = reader.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let hash = hasher.finalize().to_hex().to_string();
        Ok((hash, metadata.len()))
    }

    pub fn load_manifest(manifest_path: &Path) -> Result<ArtifactManifest, String> {
        let content = std::fs::read_to_string(manifest_path)
            .map_err(|e| format!("Failed to read manifest {}: {}", manifest_path.display(), e))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse TOML manifest: {}", e))
    }

    pub fn save_manifest(manifest: &ArtifactManifest, manifest_path: &Path) -> Result<(), String> {
        let content = toml::to_string_pretty(manifest)
            .map_err(|e| format!("Failed to serialize TOML: {}", e))?;
        std::fs::write(manifest_path, content)
            .map_err(|e| format!("Failed to write manifest {}: {}", manifest_path.display(), e))
    }

    pub fn verify(manifest: &ArtifactManifest, base_dir: &Path) -> ArtifactReport {
        let mut verified_ok = 0;
        let mut mismatched = Vec::new();
        let mut missing = Vec::new();
        let mut total_bytes = 0;

        for entry in &manifest.artifacts {
            total_bytes += entry.size_bytes;
            let full_path = base_dir.join(&entry.path);

            if !full_path.exists() {
                missing.push(format!("{} ({})", entry.name, entry.path.display()));
                continue;
            }

            match Self::compute_blake3(&full_path) {
                Ok((hash, _)) => {
                    if hash.eq_ignore_ascii_case(&entry.blake3_hash) {
                        verified_ok += 1;
                    } else {
                        mismatched.push(format!(
                            "{} (expected {}, got {})",
                            entry.name, entry.blake3_hash, hash
                        ));
                    }
                }
                Err(err) => {
                    missing.push(format!("{} (read error: {})", entry.name, err));
                }
            }
        }

        ArtifactReport {
            total_artifacts: manifest.artifacts.len(),
            total_bytes,
            verified_ok,
            mismatched,
            missing,
        }
    }
}
