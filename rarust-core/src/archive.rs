//! High-level archive open/extract/list API for rarust-core.
//!
//! Wraps `rars::Archive` and `rars::ArchiveReader` to provide a stable,
//! convenience-focused interface for the CLI layer.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::entry::Entry;
use crate::error::{RarustError, Result};
use crate::multi;
use flate2::Compression as GzipCompression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
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

/// Archive reader that supports RAR plus portable formats such as ZIP and TAR.GZ.
pub enum PortableArchive {
    /// RAR archive handled by the `rars` backend.
    Rar(Box<RarArchive>),
    /// ZIP archive.
    Zip {
        /// Archive path.
        path: PathBuf,
    },
    /// Plain TAR archive.
    Tar {
        /// Archive path.
        path: PathBuf,
    },
    /// Gzip-compressed TAR archive (`.tar.gz` / `.tgz`).
    TarGz {
        /// Archive path.
        path: PathBuf,
    },
    /// Single gzip-compressed file (`.gz`).
    Gzip {
        /// Archive path.
        path: PathBuf,
    },
}

impl PortableArchive {
    /// Open an archive, detecting the format from the file extension.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_options(path, &OpenOptions::default())
    }

    /// Open an archive with custom RAR options.
    pub fn open_with_options(path: impl AsRef<Path>, options: &OpenOptions) -> Result<Self> {
        let path = path.as_ref().to_owned();
        match detect_format_from_path(&path)? {
            ArchiveFormat::Rar5 | ArchiveFormat::Rar4 => Ok(Self::Rar(Box::new(
                RarArchive::open_with_options(path, options)?,
            ))),
            ArchiveFormat::Zip => Ok(Self::Zip { path }),
            ArchiveFormat::Tar => Ok(Self::Tar { path }),
            ArchiveFormat::TarGz => Ok(Self::TarGz { path }),
            ArchiveFormat::Gzip => Ok(Self::Gzip { path }),
        }
    }

    /// Display name for the detected archive format.
    pub fn format_name(&self) -> &'static str {
        match self {
            Self::Rar(archive) => match archive.family() {
                rars::ArchiveFamily::Rar13 => "RAR 1.3/1.4",
                rars::ArchiveFamily::Rar15To40 => "RAR 1.5-4.0",
                rars::ArchiveFamily::Rar50Plus => "RAR 5.0+",
                _ => "RAR",
            },
            Self::Zip { .. } => "ZIP",
            Self::Tar { .. } => "TAR",
            Self::TarGz { .. } => "TAR.GZ",
            Self::Gzip { .. } => "GZ",
        }
    }

    /// List all entries in the archive.
    pub fn list(&self) -> Result<Vec<Entry>> {
        match self {
            Self::Rar(archive) => archive.list(),
            Self::Zip { path } => list_zip(path),
            Self::Tar { path } => list_tar(fs::File::open(path).map_err(RarustError::Io)?, "tar"),
            Self::TarGz { path } => list_tar(
                GzDecoder::new(fs::File::open(path).map_err(RarustError::Io)?),
                "gzip",
            ),
            Self::Gzip { path } => list_gzip(path),
        }
    }

    /// Extract all entries to the destination directory.
    pub fn extract_all(&self, dest: &Path) -> Result<ExtractSummary> {
        self.extract_with_filter(dest, |_| true)
    }

    /// Extract entries matching a predicate to the destination directory.
    pub fn extract_with_filter<F>(&self, dest: &Path, filter: F) -> Result<ExtractSummary>
    where
        F: Fn(&Entry) -> bool,
    {
        match self {
            Self::Rar(archive) => archive.extract_with_filter(dest, filter),
            Self::Zip { path } => extract_zip(path, dest, filter),
            Self::Tar { path } => {
                extract_tar(fs::File::open(path).map_err(RarustError::Io)?, dest, filter)
            }
            Self::TarGz { path } => extract_tar(
                GzDecoder::new(fs::File::open(path).map_err(RarustError::Io)?),
                dest,
                filter,
            ),
            Self::Gzip { path } => extract_gzip(path, dest, filter),
        }
    }

    /// Test archive integrity by streaming extractable file contents to a sink.
    pub fn test_all(&self) -> Result<TestSummary> {
        match self {
            Self::Rar(archive) => archive.test_all(),
            Self::Zip { path } => test_zip(path),
            Self::Tar { path } => test_tar(fs::File::open(path).map_err(RarustError::Io)?),
            Self::TarGz { path } => test_tar(GzDecoder::new(
                fs::File::open(path).map_err(RarustError::Io)?,
            )),
            Self::Gzip { path } => test_gzip(path),
        }
    }
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
        let entries: Vec<Entry> = self
            .inner
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
        std::fs::create_dir_all(dest).map_err(RarustError::Io)?;

        let mut extracted = 0u64;
        let mut skipped = 0u64;
        let mut errors = 0u64;

        // Collect member names we want to keep for the extraction callback
        let keep_names: std::collections::HashSet<String> =
            filtered.iter().map(|e| e.name.clone()).collect();

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
    /// ZIP archive.
    Zip,
    /// Plain TAR archive.
    Tar,
    /// Gzip-compressed TAR archive (`.tar.gz` / `.tgz`).
    TarGz,
    /// Single gzip-compressed file (`.gz`).
    Gzip,
}

impl ArchiveFormat {
    /// Detect an archive format from a filesystem path.
    pub fn from_path(path: &Path) -> Result<Self> {
        detect_format_from_path(path)
    }
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
            ArchiveFormat::Zip => self.build_zip(dest),
            ArchiveFormat::Tar => self.build_tar(dest),
            ArchiveFormat::TarGz => self.build_tar_gz(dest),
            ArchiveFormat::Gzip => self.build_gzip(dest),
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

    fn build_zip(self, dest: &Path) -> Result<()> {
        self.ensure_portable_options("ZIP")?;
        let owned = read_builder_entries(&self.entries)?;
        let file = fs::File::create(dest).map_err(RarustError::Io)?;
        let mut writer = zip::ZipWriter::new(file);
        let options =
            zip::write::SimpleFileOptions::default().compression_method(zip_method(&self.method));

        for (name, data) in owned {
            let name = normalized_archive_name(&name)?;
            writer
                .start_file(name, options)
                .map_err(|e| RarustError::Format(format!("ZIP error: {e}")))?;
            writer.write_all(&data).map_err(RarustError::Io)?;
        }
        writer
            .finish()
            .map_err(|e| RarustError::Format(format!("ZIP error: {e}")))?;
        Ok(())
    }

    fn build_tar(self, dest: &Path) -> Result<()> {
        self.ensure_portable_options("TAR")?;
        let file = fs::File::create(dest).map_err(RarustError::Io)?;
        write_tar(file, &self.entries)
    }

    fn build_tar_gz(self, dest: &Path) -> Result<()> {
        self.ensure_portable_options("TAR.GZ")?;
        let file = fs::File::create(dest).map_err(RarustError::Io)?;
        let encoder = GzEncoder::new(file, gzip_level(&self.method));
        write_tar(encoder, &self.entries)
    }

    fn build_gzip(self, dest: &Path) -> Result<()> {
        self.ensure_portable_options("GZ")?;
        if self.entries.len() != 1 {
            return Err(RarustError::Unsupported(
                "GZ output supports exactly one input file; use tar.gz for multiple files"
                    .to_string(),
            ));
        }
        let entry = &self.entries[0];
        let name = entry
            .archive_name
            .clone()
            .or_else(|| {
                entry
                    .source
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
            })
            .ok_or_else(|| RarustError::Unsupported("entry has no usable name".to_string()))?;
        let mut encoder = flate2::GzBuilder::new().filename(name).write(
            fs::File::create(dest).map_err(RarustError::Io)?,
            gzip_level(&self.method),
        );
        let mut input = fs::File::open(&entry.source).map_err(RarustError::Io)?;
        std::io::copy(&mut input, &mut encoder).map_err(RarustError::Io)?;
        encoder.finish().map_err(RarustError::Io)?;
        Ok(())
    }

    fn ensure_portable_options(&self, format: &str) -> Result<()> {
        if self.password.is_some() {
            return Err(RarustError::Unsupported(format!(
                "{format} creation does not support encryption"
            )));
        }
        if self.header_encrypt {
            return Err(RarustError::Unsupported(format!(
                "{format} creation does not support header encryption"
            )));
        }
        if self.volume_size.is_some() {
            return Err(RarustError::Unsupported(format!(
                "{format} creation does not support multi-volume splitting"
            )));
        }
        if self.recovery_percent.is_some() {
            return Err(RarustError::Unsupported(format!(
                "{format} creation does not support recovery records"
            )));
        }
        if self.solid {
            return Err(RarustError::Unsupported(format!(
                "{format} creation does not support solid compression"
            )));
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
            .ok_or_else(|| RarustError::Unsupported("entry has no usable name".to_string()))?;
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

fn detect_format_from_path(path: &Path) -> Result<ArchiveFormat> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Ok(ArchiveFormat::TarGz)
    } else if ext == "zip" {
        Ok(ArchiveFormat::Zip)
    } else if ext == "tar" {
        Ok(ArchiveFormat::Tar)
    } else if ext == "gz" {
        Ok(ArchiveFormat::Gzip)
    } else if ext == "rar"
        || (ext.len() == 3 && ext.starts_with('r') && ext[1..].chars().all(|c| c.is_ascii_digit()))
    {
        Ok(ArchiveFormat::Rar5)
    } else {
        Err(RarustError::Unsupported(format!(
            "unsupported archive extension for {}",
            path.display()
        )))
    }
}

fn list_zip(path: &Path) -> Result<Vec<Entry>> {
    let file = fs::File::open(path).map_err(RarustError::Io)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| RarustError::Format(format!("ZIP error: {e}")))?;
    let mut entries = Vec::with_capacity(archive.len());

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| RarustError::Format(format!("ZIP error: {e}")))?;
        entries.push(generic_entry(
            file.name(),
            file.size(),
            file.compressed_size(),
            file.is_dir(),
            false,
            file.compression().to_string().as_str(),
            Some(file.crc32()),
        ));
    }

    Ok(entries)
}

fn extract_zip<F>(path: &Path, dest: &Path, filter: F) -> Result<ExtractSummary>
where
    F: Fn(&Entry) -> bool,
{
    fs::create_dir_all(dest).map_err(RarustError::Io)?;
    let file = fs::File::open(path).map_err(RarustError::Io)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| RarustError::Format(format!("ZIP error: {e}")))?;
    let total = archive.len() as u64;
    let mut extracted = 0u64;
    let mut skipped = 0u64;
    let errors = 0u64;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| RarustError::Format(format!("ZIP error: {e}")))?;
        let entry = generic_entry(
            file.name(),
            file.size(),
            file.compressed_size(),
            file.is_dir(),
            false,
            file.compression().to_string().as_str(),
            Some(file.crc32()),
        );
        if !filter(&entry) {
            skipped += 1;
            continue;
        }
        let Some(target) = entry.safe_extract_path(dest) else {
            skipped += 1;
            continue;
        };
        if entry.is_directory {
            fs::create_dir_all(&target).map_err(RarustError::Io)?;
            extracted += 1;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(RarustError::Io)?;
        }
        match fs::File::create(&target) {
            Ok(mut output) => {
                std::io::copy(&mut file, &mut output).map_err(RarustError::Io)?;
                extracted += 1;
            }
            Err(error) => {
                return Err(RarustError::Io(error));
            }
        }
    }

    Ok(ExtractSummary {
        total,
        extracted,
        skipped,
        errors,
    })
}

fn test_zip(path: &Path) -> Result<TestSummary> {
    let file = fs::File::open(path).map_err(RarustError::Io)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| RarustError::Format(format!("ZIP error: {e}")))?;
    let mut tested = 0u64;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| RarustError::Format(format!("ZIP error: {e}")))?;
        if !file.is_dir() {
            std::io::copy(&mut file, &mut std::io::sink()).map_err(RarustError::Io)?;
        }
        tested += 1;
    }
    Ok(TestSummary {
        total: archive.len() as u64,
        tested,
        failed: 0,
    })
}

fn list_tar<R: Read>(reader: R, method: &str) -> Result<Vec<Entry>> {
    let mut archive = tar::Archive::new(reader);
    let mut entries = Vec::new();
    for item in archive.entries().map_err(RarustError::Io)? {
        let entry = item.map_err(RarustError::Io)?;
        let path = entry
            .path()
            .map_err(RarustError::Io)?
            .to_string_lossy()
            .replace('\\', "/");
        let size = entry.header().size().unwrap_or(0);
        entries.push(generic_entry(
            &path,
            size,
            0,
            entry.header().entry_type().is_dir(),
            false,
            method,
            None,
        ));
    }
    Ok(entries)
}

fn extract_tar<R, F>(reader: R, dest: &Path, filter: F) -> Result<ExtractSummary>
where
    R: Read,
    F: Fn(&Entry) -> bool,
{
    fs::create_dir_all(dest).map_err(RarustError::Io)?;
    let mut archive = tar::Archive::new(reader);
    let mut total = 0u64;
    let mut extracted = 0u64;
    let mut skipped = 0u64;
    let errors = 0u64;

    for item in archive.entries().map_err(RarustError::Io)? {
        let mut tar_entry = item.map_err(RarustError::Io)?;
        total += 1;
        let path = tar_entry
            .path()
            .map_err(RarustError::Io)?
            .to_string_lossy()
            .replace('\\', "/");
        let is_dir = tar_entry.header().entry_type().is_dir();
        let is_file = tar_entry.header().entry_type().is_file();
        let entry = generic_entry(
            &path,
            tar_entry.header().size().unwrap_or(0),
            0,
            is_dir,
            false,
            "tar",
            None,
        );
        if !filter(&entry) || (!is_dir && !is_file) {
            skipped += 1;
            continue;
        }
        let Some(target) = entry.safe_extract_path(dest) else {
            skipped += 1;
            continue;
        };
        if is_dir {
            fs::create_dir_all(&target).map_err(RarustError::Io)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(RarustError::Io)?;
            }
            tar_entry.unpack(&target).map_err(RarustError::Io)?;
        }
        extracted += 1;
    }

    Ok(ExtractSummary {
        total,
        extracted,
        skipped,
        errors,
    })
}

fn test_tar<R: Read>(reader: R) -> Result<TestSummary> {
    let mut archive = tar::Archive::new(reader);
    let mut total = 0u64;
    let mut tested = 0u64;
    for item in archive.entries().map_err(RarustError::Io)? {
        let mut entry = item.map_err(RarustError::Io)?;
        total += 1;
        if entry.header().entry_type().is_file() {
            std::io::copy(&mut entry, &mut std::io::sink()).map_err(RarustError::Io)?;
        }
        tested += 1;
    }
    Ok(TestSummary {
        total,
        tested,
        failed: 0,
    })
}

fn list_gzip(path: &Path) -> Result<Vec<Entry>> {
    let mut decoder = GzDecoder::new(fs::File::open(path).map_err(RarustError::Io)?);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data).map_err(RarustError::Io)?;
    Ok(vec![generic_entry(
        &gzip_output_name(path),
        data.len() as u64,
        fs::metadata(path).map_err(RarustError::Io)?.len(),
        false,
        false,
        "gzip",
        None,
    )])
}

fn extract_gzip<F>(path: &Path, dest: &Path, filter: F) -> Result<ExtractSummary>
where
    F: Fn(&Entry) -> bool,
{
    fs::create_dir_all(dest).map_err(RarustError::Io)?;
    let entry = list_gzip(path)?.remove(0);
    if !filter(&entry) {
        return Ok(ExtractSummary {
            total: 1,
            extracted: 0,
            skipped: 1,
            errors: 0,
        });
    }
    let Some(target) = entry.safe_extract_path(dest) else {
        return Ok(ExtractSummary {
            total: 1,
            extracted: 0,
            skipped: 1,
            errors: 0,
        });
    };
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(RarustError::Io)?;
    }
    let mut decoder = GzDecoder::new(fs::File::open(path).map_err(RarustError::Io)?);
    let mut output = fs::File::create(target).map_err(RarustError::Io)?;
    std::io::copy(&mut decoder, &mut output).map_err(RarustError::Io)?;
    Ok(ExtractSummary {
        total: 1,
        extracted: 1,
        skipped: 0,
        errors: 0,
    })
}

fn test_gzip(path: &Path) -> Result<TestSummary> {
    let mut decoder = GzDecoder::new(fs::File::open(path).map_err(RarustError::Io)?);
    std::io::copy(&mut decoder, &mut std::io::sink()).map_err(RarustError::Io)?;
    Ok(TestSummary {
        total: 1,
        tested: 1,
        failed: 0,
    })
}

fn write_tar<W: Write>(writer: W, entries: &[ArchiveBuilderEntry]) -> Result<()> {
    let owned = read_builder_entries(entries)?;
    let mut builder = tar::Builder::new(writer);
    for (name, data) in owned {
        let name = normalized_archive_name(&name)?;
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, name, data.as_slice())
            .map_err(RarustError::Io)?;
    }
    builder.finish().map_err(RarustError::Io)?;
    Ok(())
}

fn normalized_archive_name(name: &[u8]) -> Result<String> {
    let name = String::from_utf8_lossy(name).replace('\\', "/");
    if name.is_empty()
        || name.starts_with('/')
        || name
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(RarustError::Unsupported(format!(
            "unsafe archive entry name: {name}"
        )));
    }
    Ok(name)
}

fn generic_entry(
    name: &str,
    size: u64,
    compressed_size: u64,
    is_directory: bool,
    is_encrypted: bool,
    method: &str,
    crc32: Option<u32>,
) -> Entry {
    let ratio = if size > 0 {
        compressed_size as f64 / size as f64
    } else {
        0.0
    };
    Entry {
        name: name.replace('\\', "/"),
        name_raw: name.as_bytes().to_vec(),
        size,
        compressed_size,
        ratio,
        is_directory,
        is_encrypted,
        is_stored: method.eq_ignore_ascii_case("stored") || method == "tar",
        is_split_before: false,
        is_split_after: false,
        modified: None,
        crc32,
        method: method.to_string(),
    }
}

fn zip_method(method: &CompressionMethod) -> zip::CompressionMethod {
    match method {
        CompressionMethod::Store => zip::CompressionMethod::Stored,
        _ => zip::CompressionMethod::Deflated,
    }
}

fn gzip_level(method: &CompressionMethod) -> GzipCompression {
    match method {
        CompressionMethod::Store => GzipCompression::none(),
        CompressionMethod::Fastest | CompressionMethod::Fast => GzipCompression::fast(),
        CompressionMethod::Best => GzipCompression::best(),
        CompressionMethod::Normal | CompressionMethod::Good => GzipCompression::default(),
    }
}

fn gzip_output_name(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output");
    file_name
        .strip_suffix(".gz")
        .unwrap_or(file_name)
        .to_string()
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
