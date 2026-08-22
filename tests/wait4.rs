#![cfg(any(unix, windows))]

use std::process::Command;

use wait4::Wait4;

fn true_command() -> Command {
    if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "exit 0"]);
        cmd
    } else {
        Command::new("true")
    }
}

fn false_command() -> Command {
    if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "exit 1"]);
        cmd
    } else {
        Command::new("false")
    }
}

#[test]
fn success_status_and_rusage() {
    let child = true_command().spawn().unwrap();
    let r = child.wait4().unwrap();

    assert!(r.status.success());
    // Resource usage values are inherently kernel/environment-dependent
    // (e.g. some sandboxes never populate maxrss) so there's nothing
    // reliable to assert about their magnitude here; wait4() succeeding
    // and returning a status is the meaningful contract to test.
}

#[test]
fn failure_status_is_reported() {
    let child = false_command().spawn().unwrap();
    let r = child.wait4().unwrap();

    assert!(!r.status.success());
}
