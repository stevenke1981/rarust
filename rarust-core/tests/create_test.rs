use std::fs;

use rarust_core::archive::{ArchiveBuilder, CompressionMethod, RarArchive};

fn temp_file(dir: &std::path::Path, name: &str, content: &[u8]) -> std::path::PathBuf {
    let p = dir.join(name);
    fs::write(&p, content).expect("write input file");
    p
}

#[test]
fn create_store_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let a = temp_file(tmp.path(), "a.txt", b"alpha");
    let b = temp_file(tmp.path(), "b.txt", b"beta beta");
    let out = tmp.path().join("out.rar");

    ArchiveBuilder::new()
        .add_file_as(&a, "a.txt")
        .add_file_as(&b, "b.txt")
        .with_method(CompressionMethod::Store)
        .build(&out)
        .expect("build archive");

    let archive = RarArchive::open(&out).expect("open created archive");
    let entries = archive.list().expect("list");
    assert_eq!(entries.len(), 2);

    let extract_dir = tmp.path().join("extract");
    let summary = archive.extract_all(&extract_dir).expect("extract");
    assert_eq!(summary.extracted, 2);
    assert_eq!(fs::read(extract_dir.join("a.txt")).expect("read a"), b"alpha");
    assert_eq!(
        fs::read(extract_dir.join("b.txt")).expect("read b"),
        b"beta beta"
    );
}

#[test]
fn create_compressed_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Repetitive data compresses well, exercising the encoder.
    let content = b"the quick brown fox jumps over the lazy dog. ".repeat(200);
    let src = temp_file(tmp.path(), "doc.txt", &content);
    let out = tmp.path().join("compressed.rar");

    ArchiveBuilder::new()
        .add_file_as(&src, "doc.txt")
        .with_method(CompressionMethod::Best)
        .build(&out)
        .expect("build compressed archive");

    let archive = RarArchive::open(&out).expect("open");
    let entries = archive.list().expect("list");
    assert_eq!(entries.len(), 1);
    assert!(entries[0].size >= content.len() as u64);

    let extract_dir = tmp.path().join("extract");
    let summary = archive.extract_all(&extract_dir).expect("extract");
    assert_eq!(summary.extracted, 1);
    assert_eq!(
        fs::read(extract_dir.join("doc.txt")).expect("read doc"),
        content
    );
}

/// rars 0.4.1 does not support encrypted archive creation at runtime.
/// This test verifies we get a clear `Unsupported` error rather than a panic.
#[test]
fn create_encrypted_rejected() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = temp_file(tmp.path(), "secret.txt", b"topsecret");
    let out = tmp.path().join("enc.rar");

    let err = ArchiveBuilder::new()
        .add_file_as(&src, "secret.txt")
        .with_password("pw123".to_string())
        .with_method(CompressionMethod::Store)
        .build(&out)
        .expect_err("encrypted build should be rejected");

    let msg = err.to_string();
    assert!(
        msg.contains("not supported"),
        "error should mention 'not supported': {msg}"
    );
}

#[test]
fn create_volume_split() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let data = vec![b'x'; 5000];
    let src = temp_file(tmp.path(), "big.txt", &data);
    let out = tmp.path().join("vol.rar");

    ArchiveBuilder::new()
        .add_file_as(&src, "big.txt")
        .with_volume_size(1024)
        .with_method(CompressionMethod::Store)
        .build(&out)
        .expect("build multi-volume archive");

    // Expect at least two volume files.
    let p1 = tmp.path().join("vol.part1.rar");
    let p2 = tmp.path().join("vol.part2.rar");
    assert!(p1.exists(), "vol.part1.rar should exist");
    assert!(p2.exists(), "vol.part2.rar should exist");
    assert!(
        p2.metadata().unwrap().len() < 1024 + 128,
        "part2 should be a small tail volume"
    );
}
