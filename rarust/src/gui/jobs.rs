//! Background jobs for long-running GUI archive operations.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Instant;

use rarust_core::archive::{
    ArchiveBuilder, ArchiveProgress, CompressionMethod as CoreCompressionMethod, ExtractSummary,
    OpenOptions, PortableArchive, TestSummary,
};
use rarust_core::error::{RarustError, Result};

use super::actions::create_dialog::{
    CompressionMethod as GuiCompressionMethod, CreateArchiveParams,
};

pub struct ActiveJob {
    pub receiver: Receiver<JobMessage>,
    cancel: Arc<AtomicBool>,
}

impl ActiveJob {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

pub enum JobMessage {
    Progress(JobProgress),
    Finished(JobOutcome),
}

pub struct JobProgress {
    pub title: String,
    pub status: String,
    pub progress: Option<f64>,
    pub total_bytes: u64,
    pub processed_bytes: u64,
    pub speed_bytes_per_sec: f64,
}

pub enum JobOutcome {
    Extracted {
        summary: ExtractSummary,
        dest: PathBuf,
    },
    Tested(TestSummary),
    Created {
        archive_path: String,
    },
    Cancelled,
    Failed(String),
}

pub fn start_extract_job(
    archive_path: String,
    password: Option<String>,
    dest: PathBuf,
    entry_name: String,
    is_directory: bool,
    title: String,
) -> ActiveJob {
    spawn_job(title, move |cancel, progress| {
        let options = OpenOptions {
            password: password.map(rarust_core::encryption::Password::from_string),
            ..OpenOptions::default()
        };
        let archive = PortableArchive::open_with_options(&archive_path, &options)?;
        let prefix = if is_directory && !entry_name.ends_with('/') {
            format!("{entry_name}/")
        } else {
            entry_name.clone()
        };
        let summary = archive.extract_with_filter_controlled(
            &dest,
            |entry| {
                if is_directory {
                    entry.name == entry_name || entry.name.starts_with(&prefix)
                } else {
                    entry.name == entry_name
                }
            },
            progress,
            || cancel.load(Ordering::Relaxed),
        )?;
        Ok(JobOutcome::Extracted { summary, dest })
    })
}

pub fn start_test_job(archive_path: String, password: Option<String>, title: String) -> ActiveJob {
    spawn_job(title, move |cancel, progress| {
        let options = OpenOptions {
            password: password.map(rarust_core::encryption::Password::from_string),
            ..OpenOptions::default()
        };
        let archive = PortableArchive::open_with_options(&archive_path, &options)?;
        let summary = archive.test_all_controlled(progress, || cancel.load(Ordering::Relaxed))?;
        Ok(JobOutcome::Tested(summary))
    })
}

pub fn start_create_job(params: CreateArchiveParams, title: String) -> ActiveJob {
    spawn_job(title, move |cancel, mut progress| {
        if cancel.load(Ordering::Relaxed) {
            return Err(RarustError::Cancelled);
        }

        progress(ArchiveProgress {
            current_entry: "Preparing input files".to_string(),
            total_entries: params.source_paths.len() as u64,
            completed_entries: 0,
            total_bytes: source_total_bytes(&params.source_paths),
            processed_bytes: 0,
        });

        let mut builder = ArchiveBuilder::new().with_method(map_method(params.method));
        if let Some(pwd) = &params.password {
            builder = builder.with_password(pwd.clone());
            if params.encrypt_filenames {
                builder = builder.with_header_encrypt(true);
            }
        }
        if params.split_mb > 0 {
            builder = builder.with_volume_size(params.split_mb * 1024 * 1024);
        }
        for (idx, src) in params.source_paths.iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                return Err(RarustError::Cancelled);
            }
            progress(ArchiveProgress {
                current_entry: src.clone(),
                total_entries: params.source_paths.len() as u64,
                completed_entries: idx as u64,
                total_bytes: 0,
                processed_bytes: 0,
            });
            builder = builder.add_file(src);
        }

        progress(ArchiveProgress {
            current_entry: "Writing archive".to_string(),
            total_entries: params.source_paths.len() as u64,
            completed_entries: params.source_paths.len() as u64,
            total_bytes: 0,
            processed_bytes: 0,
        });
        builder.build(&params.archive_path)?;
        Ok(JobOutcome::Created {
            archive_path: params.archive_path,
        })
    })
}

fn spawn_job<F>(title: String, work: F) -> ActiveJob
where
    F: FnOnce(Arc<AtomicBool>, Box<dyn FnMut(ArchiveProgress) + Send>) -> Result<JobOutcome>
        + Send
        + 'static,
{
    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let worker_cancel = cancel.clone();
    thread::spawn(move || {
        let started = Instant::now();
        let progress_title = title.clone();
        let progress_tx = tx.clone();
        let progress = Box::new(move |p: ArchiveProgress| {
            let elapsed = started.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 && p.processed_bytes > 0 {
                p.processed_bytes as f64 / elapsed
            } else {
                0.0
            };
            let ratio = progress_ratio(&p);
            let status = if p.current_entry.is_empty() {
                progress_title.clone()
            } else {
                p.current_entry.clone()
            };
            let _ = progress_tx.send(JobMessage::Progress(JobProgress {
                title: progress_title.clone(),
                status,
                progress: ratio,
                total_bytes: p.total_bytes,
                processed_bytes: p.processed_bytes,
                speed_bytes_per_sec: speed,
            }));
        });

        let outcome = match work(worker_cancel, progress) {
            Ok(outcome) => outcome,
            Err(RarustError::Cancelled) => JobOutcome::Cancelled,
            Err(error) => JobOutcome::Failed(error.to_string()),
        };
        let _ = tx.send(JobMessage::Finished(outcome));
    });

    ActiveJob {
        receiver: rx,
        cancel,
    }
}

fn progress_ratio(progress: &ArchiveProgress) -> Option<f64> {
    if progress.total_bytes > 0 {
        Some(progress.processed_bytes as f64 / progress.total_bytes as f64)
    } else if progress.total_entries > 0 {
        Some(progress.completed_entries as f64 / progress.total_entries as f64)
    } else {
        None
    }
}

fn map_method(method: GuiCompressionMethod) -> CoreCompressionMethod {
    match method {
        GuiCompressionMethod::Store => CoreCompressionMethod::Store,
        GuiCompressionMethod::Fastest => CoreCompressionMethod::Fastest,
        GuiCompressionMethod::Fast => CoreCompressionMethod::Fast,
        GuiCompressionMethod::Normal | GuiCompressionMethod::Tqt => CoreCompressionMethod::Normal,
        GuiCompressionMethod::Good => CoreCompressionMethod::Good,
        GuiCompressionMethod::Best => CoreCompressionMethod::Best,
    }
}

fn source_total_bytes(paths: &[String]) -> u64 {
    paths
        .iter()
        .map(Path::new)
        .filter_map(|path| std::fs::metadata(path).ok())
        .map(|meta| meta.len())
        .sum()
}
