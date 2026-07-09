//! Header-encrypted archive create + read roundtrip.

use std::fs;

use rarust_core::archive::{
    ArchiveBuilder, ArchiveFormat, CompressionMethod, OpenOptions, RarArchive,
};
use rarust_core::error::RarustError;

fn temp_file(dir: &std::path::Path, name: &str, content: &[u8]) -> std::path::PathBuf {
    let p = dir.join(name);
    fs::write(&p, content).expect("write input file");
    p
}

#[test]
fn create_header_encrypted_requires_password() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = temp_file(tmp.path(), "a.txt", b"data");
    let out = tmp.path().join("hp.rar");

    let err = ArchiveBuilder::new()
        .add_file_as(&src, "a.txt")
        .with_header_encrypt(true)
        .build(&out)
        .expect_err("header encrypt without password");

    assert!(matches!(err, RarustError::Unsupported(_)));
}

#[test]
fn create_header_encrypted_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = temp_file(tmp.path(), "hidden.txt", b"secret names");
    let out = tmp.path().join("hp.rar");

    ArchiveBuilder::new()
        .add_file_as(&src, "hidden.txt")
        .with_password("hpw")
        .with_header_encrypt(true)
        .with_method(CompressionMethod::Store)
        .build(&out)
        .expect("build header-encrypted archive");

    assert!(
        RarArchive::open(&out).is_err(),
        "header-encrypted archive must require a password to open"
    );

    let archive = RarArchive::open_with_options(
        &out,
        &OpenOptions {
            password: Some(rarust_core::encryption::Password::from_string(
                "hpw".to_string(),
            )),
            ..OpenOptions::default()
        },
    )
    .expect("open with password");

    let entries = archive.list().expect("list with password");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "hidden.txt");

    let extract_dir = tmp.path().join("out");
    archive.extract_all(&extract_dir).expect("extract");
    assert_eq!(
        fs::read(extract_dir.join("hidden.txt")).expect("read"),
        b"secret names"
    );
}

#[test]
fn create_rar4_header_encrypted_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = temp_file(tmp.path(), "legacy-secret.txt", b"rar4 header payload");
    let out = tmp.path().join("legacy-hp.rar");

    ArchiveBuilder::new()
        .with_format(ArchiveFormat::Rar4)
        .add_file_as(&src, "legacy-secret.txt")
        .with_password("hpw")
        .with_header_encrypt(true)
        .with_method(CompressionMethod::Store)
        .build(&out)
        .expect("build rar4 header-encrypted archive");

    assert!(
        RarArchive::open(&out).is_err(),
        "RAR4 header-encrypted archive must require a password to open"
    );

    let archive = RarArchive::open_with_options(
        &out,
        &OpenOptions {
            password: Some(rarust_core::encryption::Password::from_string(
                "hpw".to_string(),
            )),
            ..OpenOptions::default()
        },
    )
    .expect("open rar4 header-encrypted archive");

    let extract_dir = tmp.path().join("out");
    archive.extract_all(&extract_dir).expect("extract");
    assert_eq!(
        fs::read(extract_dir.join("legacy-secret.txt")).expect("read"),
        b"rar4 header payload"
    );
}
