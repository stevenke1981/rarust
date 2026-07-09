use std::fs;
use std::process::Command;

fn rar50_stored_archive() -> Vec<u8> {
    rars::rar50::Rar50Writer::new(rars::rar50::WriterOptions::new(
        rars::ArchiveVersion::Rar50,
        rars::FeatureSet::store_only(),
    ))
    .stored_entries(&[
        rars::rar50::StoredEntry {
            name: b"hello.txt",
            data: b"hello rarust cli\n",
            mtime: Some(0),
            attributes: 0x20,
            host_os: 3,
        },
        rars::rar50::StoredEntry {
            name: b"nested/world.txt",
            data: b"nested rarust cli\n",
            mtime: Some(0),
            attributes: 0x20,
            host_os: 3,
        },
    ])
    .finish()
    .expect("fixture archive should be generated")
}

fn write_fixture(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let archive = dir.path().join("fixture.rar");
    fs::write(&archive, rar50_stored_archive()).expect("write fixture archive");
    archive
}

#[test]
fn cli_list_json_smoke() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let archive = write_fixture(&tmp);

    let output = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--json")
        .arg("list")
        .arg(&archive)
        .output()
        .expect("run rarust list");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello.txt"));
    assert!(stdout.contains("nested/world.txt"));
}

#[test]
fn cli_list_tree_smoke() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let archive = write_fixture(&tmp);

    let output = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("list")
        .arg(&archive)
        .arg("--tree")
        .output()
        .expect("run rarust list tree");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("├── hello.txt"), "stdout: {stdout}");
    assert!(stdout.contains("└── nested/"), "stdout: {stdout}");
    assert!(stdout.contains("    └── world.txt"), "stdout: {stdout}");
    assert!(!stdout.contains("nested/world.txt"), "stdout: {stdout}");
}

#[test]
fn cli_test_json_smoke() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let archive = write_fixture(&tmp);

    let output = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--json")
        .arg("test")
        .arg(&archive)
        .output()
        .expect("run rarust test");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"tested\": 2"));
    assert!(stdout.contains("\"failed\": 0"));
}

#[test]
fn cli_test_progress_smoke() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let archive = write_fixture(&tmp);

    let output = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("test")
        .arg(&archive)
        .output()
        .expect("run rarust test with progress enabled");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Test completed: 2 entries OK, 0 failed"));
}

#[test]
fn cli_test_quiet_subcommand_suppresses_stdout() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let archive = write_fixture(&tmp);

    let output = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("test")
        .arg(&archive)
        .arg("-q")
        .output()
        .expect("run rarust test quiet");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "quiet test stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn cli_extract_include_filter_smoke() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let archive = write_fixture(&tmp);
    let out_dir = tmp.path().join("out");

    let output = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--no-progress")
        .arg("extract")
        .arg(&archive)
        .arg(&out_dir)
        .arg("--include")
        .arg("nested/")
        .output()
        .expect("run rarust extract");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!out_dir.join("hello.txt").exists());
    assert_eq!(
        fs::read_to_string(out_dir.join("nested/world.txt")).expect("read extracted file"),
        "nested rarust cli\n"
    );
}

#[test]
fn cli_extract_dry_run_honors_include_filter_smoke() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let archive = write_fixture(&tmp);
    let out_dir = tmp.path().join("out-dry-run");

    let output = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("extract")
        .arg(&archive)
        .arg(&out_dir)
        .arg("--dry-run")
        .arg("--include")
        .arg("nested/")
        .output()
        .expect("run rarust extract dry-run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Would extract 1 files"), "stdout: {stdout}");
    assert!(stdout.contains("nested/world.txt"), "stdout: {stdout}");
    assert!(!stdout.contains("hello.txt"), "stdout: {stdout}");
    assert!(!out_dir.exists(), "dry-run should not create destination");
}

#[test]
fn cli_extract_progress_smoke() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let archive = write_fixture(&tmp);
    let out_dir = tmp.path().join("out-progress");

    let output = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("extract")
        .arg(&archive)
        .arg(&out_dir)
        .output()
        .expect("run rarust extract with progress enabled");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(out_dir.join("hello.txt")).expect("read extracted file"),
        "hello rarust cli\n"
    );
    assert_eq!(
        fs::read_to_string(out_dir.join("nested/world.txt")).expect("read extracted file"),
        "nested rarust cli\n"
    );
}

#[test]
fn cli_extract_json_suppresses_progress_smoke() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let archive = write_fixture(&tmp);
    let out_dir = tmp.path().join("out-json");

    let output = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--json")
        .arg("extract")
        .arg(&archive)
        .arg(&out_dir)
        .output()
        .expect("run rarust extract json");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"extracted\": 2"));
    assert!(
        !stdout.contains("Extracting"),
        "JSON stdout must not contain progress output: {stdout}"
    );
}

#[test]
fn cli_create_header_encrypt_requires_password_to_open() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let input = tmp.path().join("hidden.txt");
    fs::write(&input, b"header encrypted payload\n").expect("write input");

    let archive = tmp.path().join("hp.rar");
    let create = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--no-progress")
        .arg("create")
        .arg(&archive)
        .arg(&input)
        .arg("--method")
        .arg("store")
        .arg("-p")
        .arg("hpw")
        .arg("--header-encrypt")
        .arg("-f")
        .output()
        .expect("run rarust create");

    assert!(
        create.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&create.stderr)
    );
    assert!(archive.exists(), "archive should be created");

    let list_no_pwd = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--no-progress")
        .arg("list")
        .arg(&archive)
        .output()
        .expect("run rarust list without password");

    assert!(
        !list_no_pwd.status.success(),
        "listing header-encrypted archive without password should fail"
    );

    let out_dir = tmp.path().join("out");
    let extract = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--no-progress")
        .arg("extract")
        .arg(&archive)
        .arg(&out_dir)
        .arg("--password")
        .arg("hpw")
        .output()
        .expect("run rarust extract");

    assert!(
        extract.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&extract.stderr)
    );
    assert_eq!(
        fs::read_to_string(out_dir.join("hidden.txt")).expect("read extracted file"),
        "header encrypted payload\n"
    );
}

#[test]
fn cli_create_extract_roundtrip() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let input = tmp.path().join("hello.txt");
    fs::write(&input, b"cli create test\n").expect("write input");

    let archive = tmp.path().join("out.rar");
    let create = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--no-progress")
        .arg("create")
        .arg(&archive)
        .arg(&input)
        .arg("--method")
        .arg("store")
        .output()
        .expect("run rarust create");

    assert!(
        create.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&create.stderr)
    );
    assert!(archive.exists(), "archive should be created");

    let out_dir = tmp.path().join("out");
    let extract = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--no-progress")
        .arg("extract")
        .arg(&archive)
        .arg(&out_dir)
        .output()
        .expect("run rarust extract");

    assert!(
        extract.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&extract.stderr)
    );
    assert_eq!(
        fs::read_to_string(out_dir.join("hello.txt")).expect("read extracted file"),
        "cli create test\n"
    );
}

#[test]
fn cli_create_zip_by_extension_roundtrip() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let input = tmp.path().join("hello.txt");
    fs::write(&input, b"cli zip create test\n").expect("write input");

    let archive = tmp.path().join("out.zip");
    let create = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--no-progress")
        .arg("create")
        .arg(&archive)
        .arg(&input)
        .arg("--method")
        .arg("best")
        .output()
        .expect("run rarust create zip");

    assert!(
        create.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&create.stderr)
    );
    let portable = rarust_core::archive::PortableArchive::open(&archive)
        .expect("created archive should be a real zip");
    assert_eq!(portable.format_name(), "ZIP");
    assert_eq!(portable.list().expect("list created zip").len(), 1);

    let out_dir = tmp.path().join("out");
    let extract = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--no-progress")
        .arg("extract")
        .arg(&archive)
        .arg(&out_dir)
        .output()
        .expect("run rarust extract zip");

    assert!(
        extract.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&extract.stderr)
    );
    assert_eq!(
        fs::read_to_string(out_dir.join("hello.txt")).expect("read extracted file"),
        "cli zip create test\n"
    );
}

#[test]
fn cli_create_tar_gz_by_extension_roundtrip() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let input = tmp.path().join("doc.txt");
    fs::write(&input, b"cli tar gz create test\n").expect("write input");

    let archive = tmp.path().join("out.tar.gz");
    let create = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--no-progress")
        .arg("create")
        .arg(&archive)
        .arg(&input)
        .output()
        .expect("run rarust create tar.gz");

    assert!(
        create.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&create.stderr)
    );
    let portable = rarust_core::archive::PortableArchive::open(&archive)
        .expect("created archive should be a real tar.gz");
    assert_eq!(portable.format_name(), "TAR.GZ");
    assert_eq!(portable.list().expect("list created tar.gz").len(), 1);

    let out_dir = tmp.path().join("out");
    let extract = Command::new(env!("CARGO_BIN_EXE_rarust"))
        .arg("--no-progress")
        .arg("extract")
        .arg(&archive)
        .arg(&out_dir)
        .output()
        .expect("run rarust extract tar.gz");

    assert!(
        extract.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&extract.stderr)
    );
    assert_eq!(
        fs::read_to_string(out_dir.join("doc.txt")).expect("read extracted file"),
        "cli tar gz create test\n"
    );
}
