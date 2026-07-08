use indicatif::{ProgressBar, ProgressStyle};
use rarust_core::archive::ArchiveProgress;

pub(crate) fn progress_bar(
    label: &'static str,
    total_entries: u64,
    total_bytes: u64,
) -> ProgressBar {
    let pb = if total_bytes > 0 {
        ProgressBar::new(total_bytes)
    } else {
        ProgressBar::new(total_entries)
    };
    let style = ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("#>-");
    pb.set_style(style);
    pb.set_message(label);
    pb
}

pub(crate) fn update_progress_bar(pb: &ProgressBar, progress: &ArchiveProgress) {
    if progress.total_bytes > 0 {
        pb.set_length(progress.total_bytes);
        pb.set_position(progress.processed_bytes.min(progress.total_bytes));
    } else if progress.total_entries > 0 {
        pb.set_length(progress.total_entries);
        pb.set_position(progress.completed_entries.min(progress.total_entries));
    }
    if !progress.current_entry.is_empty() {
        pb.set_message(progress.current_entry.clone());
    }
}
