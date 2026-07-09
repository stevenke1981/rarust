use std::fs;

use rarust_core::archive::{ExtractOptions, OpenOptions, OverwritePolicy, RarArchive};

fn rar50_stored_archive() -> Vec<u8> {
    rars::rar50::Rar50Writer::new(rars::rar50::WriterOptions::new(
        rars::ArchiveVersion::Rar50,
        rars::FeatureSet::store_only(),
    ))
    .stored_entries(&[
        rars::rar50::StoredEntry {
            name: b"hello.txt",
            data: b"hello rarust\n",
            mtime: Some(0),
            attributes: 0x20,
            host_os: 3,
        },
        rars::rar50::StoredEntry {
            name: b"nested/world.txt",
            data: b"nested rarust\n",
            mtime: Some(0),
            attributes: 0x20,
            host_os: 3,
        },
    ])
    .finish()
    .expect("fixture archive should be generated")
}

fn write_fixture() -> tempfile::NamedTempFile {
    let fixture = tempfile::NamedTempFile::new().expect("temp archive file");
    fs::write(fixture.path(), rar50_stored_archive()).expect("write fixture archive");
    fixture
}

#[test]
fn list_reads_generated_rar50_stored_archive() {
    let fixture = write_fixture();
    let archive = RarArchive::open(fixture.path()).expect("open fixture");
    let entries = archive.list().expect("list entries");

    assert_eq!(archive.family(), rarust_core::ArchiveFamily::Rar50Plus);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name, "hello.txt");
    assert_eq!(entries[0].size, b"hello rarust\n".len() as u64);
    assert_eq!(entries[1].name, "nested/world.txt");
}

#[test]
fn extract_with_filter_only_writes_matching_entries() {
    let fixture = write_fixture();
    let archive = RarArchive::open(fixture.path()).expect("open fixture");
    let out = tempfile::tempdir().expect("temp output dir");

    let summary = archive
        .extract_with_filter(out.path(), |entry| entry.name.contains("nested/"))
        .expect("extract filtered entries");

    assert_eq!(summary.total, 1);
    assert_eq!(summary.extracted, 1);
    assert!(summary.skipped >= 1);
    assert!(!out.path().join("hello.txt").exists());
    assert_eq!(
        fs::read_to_string(out.path().join("nested/world.txt")).expect("read extracted file"),
        "nested rarust\n"
    );
}

#[test]
fn test_all_streams_archive_to_sink() {
    let fixture = write_fixture();
    let archive = RarArchive::open_with_options(fixture.path(), &OpenOptions::default())
        .expect("open fixture");

    let summary = archive.test_all().expect("test archive");

    assert_eq!(summary.total, 2);
    assert_eq!(summary.tested, 2);
    assert_eq!(summary.failed, 0);
}

#[test]
fn extract_flat_writes_basenames_only() {
    let fixture = write_fixture();
    let archive = RarArchive::open(fixture.path()).expect("open fixture");
    let out = tempfile::tempdir().expect("temp output dir");

    let summary = archive
        .extract_with_options(
            out.path(),
            &ExtractOptions {
                flat: true,
                overwrite: OverwritePolicy::Overwrite,
            },
            |_| true,
        )
        .expect("flat extract");

    assert_eq!(summary.extracted, 2);
    assert!(out.path().join("hello.txt").exists());
    assert!(out.path().join("world.txt").exists());
    assert!(!out.path().join("nested").exists());
}

#[test]
fn extract_skip_does_not_overwrite_existing() {
    let fixture = write_fixture();
    let archive = RarArchive::open(fixture.path()).expect("open fixture");
    let out = tempfile::tempdir().expect("temp output dir");

    fs::write(out.path().join("hello.txt"), b"keep-me").expect("seed existing");

    let summary = archive
        .extract_with_options(
            out.path(),
            &ExtractOptions {
                flat: false,
                overwrite: OverwritePolicy::Skip,
            },
            |e| e.name == "hello.txt",
        )
        .expect("skip extract");

    assert_eq!(summary.extracted, 0);
    // skipped includes filtered-out members + the existing hello.txt
    assert!(summary.skipped >= 1);
    assert_eq!(
        fs::read_to_string(out.path().join("hello.txt")).expect("read"),
        "keep-me"
    );
}

#[test]
fn extract_rename_writes_unique_sibling() {
    let fixture = write_fixture();
    let archive = RarArchive::open(fixture.path()).expect("open fixture");
    let out = tempfile::tempdir().expect("temp output dir");

    fs::write(out.path().join("hello.txt"), b"original").expect("seed existing");

    let summary = archive
        .extract_with_options(
            out.path(),
            &ExtractOptions {
                flat: false,
                overwrite: OverwritePolicy::Rename,
            },
            |e| e.name == "hello.txt",
        )
        .expect("rename extract");

    assert_eq!(summary.extracted, 1);
    assert_eq!(
        fs::read_to_string(out.path().join("hello.txt")).expect("original"),
        "original"
    );
    assert_eq!(
        fs::read_to_string(out.path().join("hello (1).txt")).expect("renamed"),
        "hello rarust\n"
    );
}
