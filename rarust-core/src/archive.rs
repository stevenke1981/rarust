//! High-level archive open/extract/list API for rarust-core.
//!
//! Wraps `rars::Archive` and `rars::ArchiveReader` to provide a stable,
//! convenience-focused interface for the CLI layer.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::entry::Entry;
use crate::error::{RarustError, Result};
use crate::multi;
use rars::rar15_40::{
    self, FileEntry as Rar4FileEntry, StoredEntry as Rar4StoredEntry,
    WriterOptions as Rar4WriterOptions,
};
use rars::rar50::{
    CompressedEntry, EncryptedCompressedEntry, EncryptedStoredEntry, Rar50VolumeWriter,
    Rar50Writer, StoredEntry, WriterOptions,
};
use rars::{ArchiveReadOptions, ArchiveReader, ArchiveVersion, FeatureSet};

/// Options for opening an archive.
#[derive(Clone, Debug)]
pub struct OpenOptions {
    /// Optional password for encrypted archives.
    pub password: Option<String>,
    /// Whether to keep broken files on CRC error during extraction.
    pub keep_broken: bool,
    /// Memory limit for decompression dictionary (bytes).
    pub memory_limit: Option<u64>,
}

impl Default for OpenOptions {
    fn default() -> Self {
        OpenOptions {
            password: None,
            keep_broken: false,
            memory_limit: Some(256 * 1024 * 1024), // 256 MB default
        }
    }
}

/// A parsed RAR archive, ready for listing or extraction.
pub struct RarArchive {
    /// Primary volume (`part1` / `.rar`) used for listing metadata.
    inner: rars::Archive,
    /// All volumes when this is a multi-volume set (single element otherwise).
    volumes: Vec<rars::Archive>,
    /// Path this archive was opened from.
    path: PathBuf,
    /// Password for encrypted archives (stored for extraction).
    password: Option<Vec<u8>>,
}

impl RarArchive {
    /// Open a RAR archive from a filesystem path.
    ///
    /// Automatically detects and opens multi-volume sets (`.partN.rar` or `.r00`).
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_options(path, &OpenOptions::default())
    }

    /// Open a RAR archive with custom options (password, etc).
    ///
    /// When the path belongs to a multi-volume set, every part must exist on disk.
    pub fn open_with_options(path: impl AsRef<Path>, options: &OpenOptions) -> Result<Self> {
        let path = path.as_ref().to_owned();
        let password = options.password.clone();
        let volume_paths = multi::detect_volumes(&path);
        multi::ensure_volumes_exist(&volume_paths)?;

        let rars_opts = read_options_from(password.as_ref().map(|s| s.as_bytes()));
        let mut volumes = Vec::with_capacity(volume_paths.len());
        for vol_path in &volume_paths {
            volumes.push(
                ArchiveReader::read_path_with_options(vol_path, rars_opts)
                    .map_err(RarustError::Rars)?,
            );
        }

        let inner = volumes
            .first()
            .cloned()
            .ok_or_else(|| RarustError::Corrupt("empty volume set".to_string()))?;

        Ok(RarArchive {
            inner,
            volumes,
            path,
            password: password.map(|s| s.into_bytes()),
        })
    }

    /// Number of volumes in this archive set (1 for a single file).
    pub fn volume_count(&self) -> usize {
        self.volumes.len()
    }

    /// Whether this archive spans multiple volume files.
    pub fn is_multivolume(&self) -> bool {
        self.volumes.len() > 1
    }

    /// Return the archive family (RAR5, RAR4, etc).
    pub fn family(&self) -> rars::ArchiveFamily {
        self.inner.family()
    }

    /// Return the filesystem path this archive was opened from.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// List all entries in the archive.
    pub fn list(&self) -> Result<Vec<Entry>> {
        let entries: Vec<Entry> = self.inner
            .members()
            .map(|m| Entry::from_rars_meta(&m.meta))
            .collect();
        Ok(entries)
    }

    /// Extract all entries to the given destination directory.
    ///
    /// Uses `rars::Archive::extract_to` with per-entry writer callbacks.
    pub fn extract_all(&self, dest: &Path) -> Result<ExtractSummary> {
        self.extract_with_filter(dest, |_| true)
    }

    /// Extract entries matching a predicate to the given destination.
    pub fn extract_with_filter<F>(&self, dest: &Path, filter: F) -> Result<ExtractSummary>
    where
        F: Fn(&Entry) -> bool,
    {
        // Collect entries first to apply filter
        let entries = self.list()?;
        let filtered: Vec<&Entry> = entries.iter().filter(|e| filter(e)).collect();
        let total = filtered.len();

        // Create destination directory
        std::fs::create_dir_all(dest)
            .map_err(RarustError::Io)?;

        let mut extracted = 0u64;
        let mut skipped = 0u64;
        let mut errors = 0u64;

        // Collect member names we want to keep for the extraction callback
        let keep_names: std::collections::HashSet<String> = filtered
            .iter()
            .map(|e| e.name.clone())
            .collect();

        let dest_base = dest.to_owned();
        let rars_opts = read_options_from(self.password.as_deref());

        let mut open_entry = |meta: &rars::ExtractedEntryMeta| -> std::result::Result<Box<dyn Write>, rars::error::Error> {
            let name = String::from_utf8_lossy(meta.name_bytes()).to_string();
            if !keep_names.contains(&name) {
                skipped += 1;
                return Ok(Box::new(std::io::sink()) as Box<dyn Write>);
            }

            let entry = Entry::from_extracted_meta(meta);
            if let Some(target) = entry.safe_extract_path(&dest_base) {
                if let Some(parent) = target.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if meta.is_directory {
                    let _ = std::fs::create_dir_all(&target);
                    extracted += 1;
                    Ok(Box::new(std::io::sink()) as Box<dyn Write>)
                } else {
                    match std::fs::File::create(&target) {
                        Ok(file) => {
                            extracted += 1;
                            Ok(Box::new(file) as Box<dyn Write>)
                        }
                        Err(e) => {
                            errors += 1;
                            Err(rars::error::Error::from(e))
                        }
                    }
                }
            } else {
                skipped += 1;
                Ok(Box::new(std::io::sink()) as Box<dyn Write>)
            }
        };

        let extract_result = if self.is_multivolume() {
            rars::extract_volumes_to_with_options(&self.volumes, rars_opts, &mut open_entry)
        } else {
            self.inner
                .extract_to_with_options(rars_opts, &mut open_entry)
        };

        extract_result.map_err(map_rars_extract_error)?;

        Ok(ExtractSummary {
            total: total as u64,
            extracted,
            skipped,
            errors,
        })
    }

    /// Test archive integrity by streaming all extractable entries to a sink.
    ///
    /// This exercises the backend extraction path, including decompression and
    /// checksum validation, without writing output files to disk.
    pub fn test_all(&self) -> Result<TestSummary> {
        let entries = self.list()?;
        let total = entries.len() as u64;
        let mut tested = 0u64;
        let rars_opts = read_options_from(self.password.as_deref());

        let mut count_entry = |_meta: &rars::ExtractedEntryMeta| -> std::result::Result<Box<dyn Write>, rars::error::Error> {
            tested += 1;
            Ok(Box::new(std::io::sink()) as Box<dyn Write>)
        };

        let test_result = if self.is_multivolume() {
            rars::extract_volumes_to_with_options(&self.volumes, rars_opts, &mut count_entry)
        } else {
            self.inner
                .extract_to_with_options(rars_opts, &mut count_entry)
        };

        test_result.map_err(map_rars_extract_error)?;

        Ok(TestSummary {
            total,
            tested,
            failed: 0,
        })
    }

    /// Return the underlying rars archive (for advanced usage).
    pub fn into_inner(self) -> rars::Archive {
        self.inner
    }
}

/// Summary of an extraction operation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtractSummary {
    /// Total files matched by filter.
    pub total: u64,
    /// Files successfully extracted.
    pub extracted: u64,
    /// Files skipped (filtered out, unsafe paths, etc).
    pub skipped: u64,
    /// Errors encountered.
    pub errors: u64,
}

/// Summary of an archive integrity test operation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TestSummary {
    /// Total entries discovered in the archive listing.
    pub total: u64,
    /// Entries streamed through the extractor successfully.
    pub tested: u64,
    /// Entries that failed validation.
    pub failed: u64,
}

/// Output archive format for creation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// RAR 5.x (default).
    #[default]
    Rar5,
    /// RAR 4.x (legacy, via the RAR 1.5–4.0 writer).
    Rar4,
}

/// Builder for creating new RAR archives.
///
/// Wraps the `rars` RAR5 and RAR4 writers (stored / compressed / encrypted / multi-volume).
pub struct ArchiveBuilder {
    entries: Vec<ArchiveBuilderEntry>,
    format: ArchiveFormat,
    password: Option<String>,
    method: CompressionMethod,
    solid: bool,
    volume_size: Option<u64>,
    recovery_percent: Option<u32>,
    header_encrypt: bool,
}

struct ArchiveBuilderEntry {
    source: PathBuf,
    archive_name: Option<String>,
}

/// Compression method / level.
#[derive(Clone, Debug, Default)]
pub enum CompressionMethod {
    /// Normal compression level (RAR m3).
    #[default]
    Normal,
    /// Store files without compression (RAR m0).
    Store,
    /// Fastest compression level (RAR m1).
    Fastest,
    /// Fast compression level (RAR m2).
    Fast,
    /// Good compression level (RAR m4).
    Good,
    /// Best compression level (RAR m5).
    Best,
}

impl CompressionMethod {
    /// Return a display label for this compression method.
    pub fn name(&self) -> &str {
        match self {
            CompressionMethod::Store => "m0 store",
            CompressionMethod::Fastest => "m1 fastest",
            CompressionMethod::Fast => "m2 fast",
            CompressionMethod::Normal => "m3 normal",
            CompressionMethod::Good => "m4 good",
            CompressionMethod::Best => "m5 best",
        }
    }
}

impl ArchiveBuilder {
    /// Create a new builder with default options (RAR5).
    pub fn new() -> Self {
        ArchiveBuilder {
            entries: Vec::new(),
            format: ArchiveFormat::Rar5,
            password: None,
            method: CompressionMethod::Normal,
            solid: false,
            volume_size: None,
            recovery_percent: None,
            header_encrypt: false,
        }
    }

    /// Set the output archive format (`Rar5` or `Rar4`).
    pub fn with_format(mut self, format: ArchiveFormat) -> Self {
        self.format = format;
        self
    }

    /// Add a file or directory to the archive.
    pub fn add_file(mut self, path: impl AsRef<Path>) -> Self {
        self.entries.push(ArchiveBuilderEntry {
            source: path.as_ref().to_owned(),
            archive_name: None,
        });
        self
    }

    /// Add a file with a custom archive name.
    pub fn add_file_as(mut self, path: impl AsRef<Path>, archive_name: impl Into<String>) -> Self {
        self.entries.push(ArchiveBuilderEntry {
            source: path.as_ref().to_owned(),
            archive_name: Some(archive_name.into()),
        });
        self
    }

    /// Set encryption password.
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set compression method.
    pub fn with_method(mut self, method: CompressionMethod) -> Self {
        self.method = method;
        self
    }

    /// Enable solid compression.
    pub fn solid(mut self, enabled: bool) -> Self {
        self.solid = enabled;
        self
    }

    /// Set volume size (for multi-volume archives).
    pub fn with_volume_size(mut self, size: u64) -> Self {
        self.volume_size = Some(size);
        self
    }

    /// Set recovery record percentage (N% of archive size).
    pub fn with_recovery_percent(mut self, percent: u32) -> Self {
        self.recovery_percent = Some(percent);
        self
    }

    /// Enable header (name) encryption. Requires a password to be set.
    pub fn with_header_encrypt(mut self, enabled: bool) -> Self {
        self.header_encrypt = enabled;
        self
    }

    /// Map the chosen method to a numeric RAR compression level.
    fn method_level(&self) -> u8 {
        match self.method {
            CompressionMethod::Store => 0,
            CompressionMethod::Fastest => 1,
            CompressionMethod::Fast => 2,
            CompressionMethod::Normal => 3,
            CompressionMethod::Good => 4,
            CompressionMethod::Best => 5,
        }
    }

    /// Build the archive to the given output path.
    pub fn build(self, dest: impl AsRef<Path>) -> Result<()> {
        let dest = dest.as_ref();
        if self.entries.is_empty() {
            return Err(RarustError::Unsupported(
                "no input files added to the archive".to_string(),
            ));
        }
        if self.header_encrypt && self.password.is_none() {
            return Err(RarustError::Unsupported(
                "header encryption requires a password".to_string(),
            ));
        }

        match self.format {
            ArchiveFormat::Rar5 => self.build_rar5(dest),
            ArchiveFormat::Rar4 => self.build_rar4(dest),
        }
    }

    fn build_rar5(self, dest: &Path) -> Result<()> {
        let owned = read_builder_entries(&self.entries)?;
        let host_os = if cfg!(windows) { 0u64 } else { 1u64 };
        let is_store = matches!(self.method, CompressionMethod::Store);
        let encrypted = self.password.is_some();

        let mut features = FeatureSet::store_only();
        features.solid = self.solid;
        features.header_encryption = self.header_encrypt;
        if encrypted {
            features.file_encryption = true;
        }

        let mut opts = WriterOptions::new(ArchiveVersion::Rar50, features);
        if !is_store {
            opts = opts.with_compression_level(self.method_level());
        }

        let pwd = self.password.as_deref();

        if let Some(vol_size) = self.volume_size {
            let mut vw = Rar50VolumeWriter::new(opts).max_payload_per_volume(vol_size as usize);
            if let Some(r) = self.recovery_percent {
                vw = vw.recovery_percent(Some(r as u64));
            }
            let volumes = if encrypted {
                let password = pwd.expect("password checked above");
                if is_store {
                    if owned.len() != 1 {
                        return Err(RarustError::Unsupported(
                            "RAR5 encrypted multi-volume currently supports one file per set"
                                .to_string(),
                        ));
                    }
                    let (n, d) = &owned[0];
                    vw.encrypted_stored_entry(EncryptedStoredEntry {
                        name: n,
                        data: d,
                        mtime: None,
                        attributes: 0x20,
                        host_os,
                        password: password.as_bytes(),
                    })
                    .finish()
                } else {
                    let enc: Vec<EncryptedCompressedEntry> = owned
                        .iter()
                        .map(|(n, d)| EncryptedCompressedEntry {
                            name: n,
                            data: d,
                            mtime: None,
                            attributes: 0x20,
                            host_os,
                            password: password.as_bytes(),
                        })
                        .collect();
                    vw.encrypted_compressed_entries(&enc).finish()
                }
            } else if is_store {
                let stored: Vec<StoredEntry> = owned
                    .iter()
                    .map(|(n, d)| StoredEntry {
                        name: n,
                        data: d,
                        mtime: None,
                        attributes: 0x20,
                        host_os,
                    })
                    .collect();
                for e in &stored {
                    vw = vw.stored_entry(*e);
                }
                vw.finish()
            } else {
                let compressed: Vec<CompressedEntry> = owned
                    .iter()
                    .map(|(n, d)| CompressedEntry {
                        name: n,
                        data: d,
                        mtime: None,
                        attributes: 0x20,
                        host_os,
                    })
                    .collect();
                vw.compressed_entries(&compressed).finish()
            }
            .map_err(RarustError::Rars)?;
            write_rar5_volume_files(dest, &volumes)?;
        } else {
            let bytes = if encrypted {
                let password = pwd.expect("password checked above");
                let mut w = Rar50Writer::new(opts);
                if let Some(r) = self.recovery_percent {
                    w = w.recovery_percent(Some(r as u64));
                }
                if is_store {
                    let enc: Vec<EncryptedStoredEntry> = owned
                        .iter()
                        .map(|(n, d)| EncryptedStoredEntry {
                            name: n,
                            data: d,
                            mtime: None,
                            attributes: 0x20,
                            host_os,
                            password: password.as_bytes(),
                        })
                        .collect();
                    w.encrypted_stored_entries(&enc).finish()
                } else {
                    let enc: Vec<EncryptedCompressedEntry> = owned
                        .iter()
                        .map(|(n, d)| EncryptedCompressedEntry {
                            name: n,
                            data: d,
                            mtime: None,
                            attributes: 0x20,
                            host_os,
                            password: password.as_bytes(),
                        })
                        .collect();
                    w.encrypted_compressed_entries(&enc).finish()
                }
            } else {
                let mut w = Rar50Writer::new(opts);
                if let Some(r) = self.recovery_percent {
                    w = w.recovery_percent(Some(r as u64));
                }
                if is_store {
                    let stored: Vec<StoredEntry> = owned
                        .iter()
                        .map(|(n, d)| StoredEntry {
                            name: n,
                            data: d,
                            mtime: None,
                            attributes: 0x20,
                            host_os,
                        })
                        .collect();
                    w.stored_entries(&stored).finish()
                } else {
                    let compressed: Vec<CompressedEntry> = owned
                        .iter()
                        .map(|(n, d)| CompressedEntry {
                            name: n,
                            data: d,
                            mtime: None,
                            attributes: 0x20,
                            host_os,
                        })
                        .collect();
                    w.compressed_entries(&compressed).finish()
                }
            }
            .map_err(RarustError::Rars)?;
            fs::write(dest, bytes).map_err(RarustError::Io)?;
        }

        Ok(())
    }

    fn build_rar4(self, dest: &Path) -> Result<()> {
        let owned = read_builder_entries(&self.entries)?;
        let host_os: u8 = if cfg!(windows) { 0 } else { 1 };
        let is_store = matches!(self.method, CompressionMethod::Store);
        let pwd_bytes = self.password.as_deref().map(str::as_bytes);

        let mut features = FeatureSet::store_only();
        features.solid = self.solid;
        features.header_encryption = self.header_encrypt;
        if pwd_bytes.is_some() {
            features.file_encryption = true;
        }

        let mut opts = Rar4WriterOptions::new(ArchiveVersion::Rar40, features);
        if !is_store {
            opts = opts.with_compression_level(self.method_level());
        }

        if let Some(vol_size) = self.volume_size {
            if owned.len() != 1 {
                return Err(RarustError::Unsupported(
                    "RAR4 multi-volume splitting supports exactly one input file".to_string(),
                ));
            }
            let (name, data) = &owned[0];
            let volumes = if is_store {
                rar15_40::write_stored_volumes(
                    Rar4StoredEntry {
                        name,
                        data,
                        file_time: 0,
                        file_attr: 0x20,
                        host_os,
                        password: pwd_bytes,
                        file_comment: None,
                    },
                    opts,
                    vol_size as usize,
                )
            } else {
                rar15_40::write_compressed_volumes(
                    Rar4FileEntry {
                        name,
                        data,
                        file_time: 0,
                        file_attr: 0x20,
                        host_os,
                        password: pwd_bytes,
                        file_comment: None,
                    },
                    opts,
                    vol_size as usize,
                )
            }
            .map_err(RarustError::Rars)?;
            write_rar4_volume_files(dest, &volumes)?;
        } else if is_store {
            let stored: Vec<Rar4StoredEntry> = owned
                .iter()
                .map(|(n, d)| Rar4StoredEntry {
                    name: n,
                    data: d,
                    file_time: 0,
                    file_attr: 0x20,
                    host_os,
                    password: pwd_bytes,
                    file_comment: None,
                })
                .collect();
            let bytes = rar15_40::write_stored_archive(&stored, opts).map_err(RarustError::Rars)?;
            fs::write(dest, bytes).map_err(RarustError::Io)?;
        } else {
            let files: Vec<Rar4FileEntry> = owned
                .iter()
                .map(|(n, d)| Rar4FileEntry {
                    name: n,
                    data: d,
                    file_time: 0,
                    file_attr: 0x20,
                    host_os,
                    password: pwd_bytes,
                    file_comment: None,
                })
                .collect();
            let bytes =
                rar15_40::write_compressed_archive(&files, opts).map_err(RarustError::Rars)?;
            fs::write(dest, bytes).map_err(RarustError::Io)?;
        }

        Ok(())
    }
}

impl Default for ArchiveBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Read all builder entries from disk into owned name/data buffers.
fn read_builder_entries(entries: &[ArchiveBuilderEntry]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let mut out = Vec::with_capacity(entries.len());
    for e in entries {
        let data = fs::read(&e.source).map_err(RarustError::Io)?;
        let name = e
            .archive_name
            .clone()
            .or_else(|| {
                e.source
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
            })
            .ok_or_else(|| {
                RarustError::Unsupported("entry has no usable name".to_string())
            })?;
        out.push((name.into_bytes(), data));
    }
    Ok(out)
}

fn read_options_from(password: Option<&[u8]>) -> ArchiveReadOptions<'_> {
    match password {
        Some(pwd) => ArchiveReadOptions::with_password(pwd),
        None => ArchiveReadOptions::default(),
    }
}

fn map_rars_extract_error(error: rars::error::Error) -> RarustError {
    if matches!(error, rars::error::Error::WrongPasswordOrCorruptData) {
        RarustError::WrongPassword
    } else {
        RarustError::Rars(error)
    }
}

/// Write RAR5 multi-volume payloads to `dest.part1.rar`, `dest.part2.rar`, ...
fn write_rar5_volume_files(dest: &Path, volumes: &[Vec<u8>]) -> Result<()> {
    let base = dest.to_string_lossy().to_string();
    let stem = if base.to_ascii_lowercase().ends_with(".rar") {
        &base[..base.len() - 4]
    } else {
        base.as_str()
    };
    for (i, data) in volumes.iter().enumerate() {
        let name = format!("{stem}.part{}.rar", i + 1);
        fs::write(&name, data).map_err(RarustError::Io)?;
    }
    Ok(())
}

/// Write RAR4 multi-volume payloads to `dest.rar`, `dest.r00`, `dest.r01`, ...
fn write_rar4_volume_files(dest: &Path, volumes: &[Vec<u8>]) -> Result<()> {
    let parent = dest.parent().unwrap_or_else(|| Path::new("."));
    let stem = dest
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("archive");
    for (i, data) in volumes.iter().enumerate() {
        let name = if i == 0 {
            format!("{stem}.rar")
        } else {
            format!("{stem}.r{:02}", i - 1)
        };
        fs::write(parent.join(name), data).map_err(RarustError::Io)?;
    }
    Ok(())
}
