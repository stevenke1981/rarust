use std::fs;

use rarust_core::archive::{ArchiveBuilder, ArchiveFormat, CompressionMethod, OpenOptions, RarArchive};

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

#[test]
fn create_encrypted_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = temp_file(tmp.path(), "secret.txt", b"topsecret payload");
    let out = tmp.path().join("enc.rar");

    ArchiveBuilder::new()
        .add_file_as(&src, "secret.txt")
        .with_password("pw123")
        .with_method(CompressionMethod::Store)
        .build(&out)
        .expect("build encrypted archive");

    let wrong_result = RarArchive::open_with_options(
        &out,
        &OpenOptions {
            password: Some("wrong".to_string()),
            ..OpenOptions::default()
        },
    );
    match wrong_result {
        Err(_) => {}
        Ok(wrong_pwd) => {
            assert!(
                wrong_pwd.extract_all(&tmp.path().join("bad")).is_err(),
                "wrong password should fail extraction"
            );
        }
    }

    let archive = RarArchive::open_with_options(
        &out,
        &OpenOptions {
            password: Some("pw123".to_string()),
            ..OpenOptions::default()
        },
    )
    .expect("open with password");

    let extract_dir = tmp.path().join("extract");
    let summary = archive.extract_all(&extract_dir).expect("extract encrypted");
    assert_eq!(summary.extracted, 1);
    assert_eq!(
        fs::read(extract_dir.join("secret.txt")).expect("read secret"),
        b"topsecret payload"
    );
}

#[test]
fn create_rar4_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = temp_file(tmp.path(), "legacy.txt", b"rar4 content here");
    let out = tmp.path().join("legacy.rar");

    ArchiveBuilder::new()
        .with_format(ArchiveFormat::Rar4)
        .add_file_as(&src, "legacy.txt")
        .with_method(CompressionMethod::Store)
        .build(&out)
        .expect("build rar4 archive");

    let archive = RarArchive::open(&out).expect("open rar4");
    assert_eq!(archive.family(), rarust_core::ArchiveFamily::Rar15To40);

    let extract_dir = tmp.path().join("extract");
    archive.extract_all(&extract_dir).expect("extract rar4");
    assert_eq!(
        fs::read(extract_dir.join("legacy.txt")).expect("read legacy"),
        b"rar4 content here"
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

    let p1 = tmp.path().join("vol.part1.rar");
    let p2 = tmp.path().join("vol.part2.rar");
    assert!(p1.exists(), "vol.part1.rar should exist");
    assert!(p2.exists(), "vol.part2.rar should exist");
    assert!(
        p2.metadata().unwrap().len() < 1024 + 128,
        "part2 should be a small tail volume"
    );
}

#[test]
fn multivolume_read_extract_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let data = vec![b'A'; 8000];
    let src = temp_file(tmp.path(), "payload.bin", &data);
    let out = tmp.path().join("mv.rar");

    ArchiveBuilder::new()
        .add_file_as(&src, "payload.bin")
        .with_volume_size(1024)
        .with_method(CompressionMethod::Store)
        .build(&out)
        .expect("build volumes");

    let archive = RarArchive::open(&tmp.path().join("mv.part2.rar"))
        .expect("open from middle volume path");
    assert!(archive.is_multivolume());
    assert!(archive.volume_count() >= 2, "expected multiple volume parts");

    let extract_dir = tmp.path().join("extract");
    archive.extract_all(&extract_dir).expect("extract multivolume");
    assert_eq!(
        fs::read(extract_dir.join("payload.bin")).expect("read payload"),
        data
    );
}