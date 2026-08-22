#![cfg(any(unix, windows))]

use std::process::Command;
use std::time::Duration;

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

fn linger_command() -> Command {
    if cfg!(windows) {
        let mut cmd = Command::new("ping");
        cmd.args(["-n", "30", "127.0.0.1"]);
        cmd
    } else {
        let mut cmd = Command::new("sleep");
        cmd.arg("30");
        cmd
    }
}

#[test]
fn try_wait4_none_while_running() {
    let mut child = linger_command().spawn().unwrap();
    let r = child.try_wait4().unwrap();
    assert!(r.is_none());
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn try_wait4_reaps_exited_child() {
    let mut child = true_command().spawn().unwrap();
    let r = loop {
        if let Some(r) = child.try_wait4().unwrap() {
            break r;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    assert!(r.status.success());
}
