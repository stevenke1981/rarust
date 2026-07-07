use std::fs;

use rarust_core::archive::{ArchiveBuilder, ArchiveFormat, CompressionMethod, PortableArchive};

fn temp_file(dir: &std::path::Path, name: &str, content: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(&path, content).expect("write input file");
    path
}

#[test]
fn zip_create_list_extract_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = temp_file(tmp.path(), "hello.txt", b"zip payload\n");
    let nested = temp_file(tmp.path(), "nested/world.txt", b"nested zip payload\n");
    let out = tmp.path().join("bundle.zip");

    ArchiveBuilder::new()
        .with_format(ArchiveFormat::Zip)
        .with_method(CompressionMethod::Best)
        .add_file_as(&src, "hello.txt")
        .add_file_as(&nested, "nested/world.txt")
        .build(&out)
        .expect("build zip archive");

    let archive = PortableArchive::open(&out).expect("open zip");
    assert_eq!(archive.format_name(), "ZIP");

    let entries = archive.list().expect("list zip");
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|entry| entry.name == "hello.txt"));
    assert!(entries.iter().any(|entry| entry.name == "nested/world.txt"));

    let extract_dir = tmp.path().join("extract");
    let summary = archive.extract_all(&extract_dir).expect("extract zip");
    assert_eq!(summary.extracted, 2);
    assert_eq!(
        fs::read_to_string(extract_dir.join("hello.txt")).expect("read hello"),
        "zip payload\n"
    );
    assert_eq!(
        fs::read_to_string(extract_dir.join("nested/world.txt")).expect("read nested"),
        "nested zip payload\n"
    );
}

#[test]
fn tar_gz_create_list_extract_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = temp_file(tmp.path(), "doc.txt", b"tar gz payload\n");
    let out = tmp.path().join("bundle.tar.gz");

    ArchiveBuilder::new()
        .with_format(ArchiveFormat::TarGz)
        .with_method(CompressionMethod::Good)
        .add_file_as(&src, "doc.txt")
        .build(&out)
        .expect("build tar.gz archive");

    let archive = PortableArchive::open(&out).expect("open tar.gz");
    assert_eq!(archive.format_name(), "TAR.GZ");

    let entries = archive.list().expect("list tar.gz");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "doc.txt");

    let extract_dir = tmp.path().join("extract");
    let summary = archive.extract_all(&extract_dir).expect("extract tar.gz");
    assert_eq!(summary.extracted, 1);
    assert_eq!(
        fs::read_to_string(extract_dir.join("doc.txt")).expect("read doc"),
        "tar gz payload\n"
    );
}
