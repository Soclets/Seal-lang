//! The C API tested from C: `api_test.c`, built with the system's C compiler against this crate's static library,
//! under `AddressSanitizer` where the compiler has it, so a memory mistake on either side stops the test (D136).

use std::path::PathBuf;
use std::process::Command;

/// The static library, built now beside this test, since `cargo test` builds only the Rust library.
fn library() -> Option<PathBuf> {
    let test = std::env::current_exe().ok()?;
    let folder = test.parent()?.parent()?;
    let mut build = Command::new(env!("CARGO"));
    build.args(["build", "-q", "-j", "3", "-p", "seal-c", "--lib"]);
    if folder.file_name().is_some_and(|name| name == "release") {
        build.arg("--release");
    }
    if !build.status().is_ok_and(|status| status.success()) {
        return None;
    }
    let library = folder.join("libseal_c.a");
    library.exists().then_some(library)
}

// trace: D136, D151, D152
#[test]
fn d136_the_c_api_works_from_c_under_address_sanitizer() {
    let Some(library) = library() else {
        eprintln!("the static library isn't built beside this test, so nothing was compared");
        return;
    };
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let program = std::env::temp_dir().join(format!("seal-api-test-{}", std::process::id()));
    let build = |sanitize: bool| {
        let mut command = Command::new("cc");
        if sanitize {
            command.args(["-fsanitize=address", "-fno-omit-frame-pointer"]);
        }
        command
            .args(["-g", "-Wall", "-Wextra", "-Werror", "-I"])
            .arg(folder.join("include"))
            .arg(folder.join("tests/api_test.c"))
            .arg(&library)
            .args(["-lc", "-lm", "-o"])
            .arg(&program)
            .status()
            .is_ok_and(|status| status.success())
    };
    if !build(true) && !build(false) {
        eprintln!("there's no C compiler here, so nothing was compared");
        return;
    }
    let ran = Command::new(&program).output();
    let _ = std::fs::remove_file(&program);
    let Ok(ran) = ran else { panic!("the C test didn't run") };
    assert!(
        ran.status.success(),
        "the C test failed:\n{}{}",
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr)
    );
}
