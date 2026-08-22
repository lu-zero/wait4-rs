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
fn success_status() {
    let child = true_command().spawn().unwrap();
    let r = child.wait4().unwrap();

    assert!(r.status.success());
}

/// The crate's core value is that rusage includes descendants the child
/// itself already waited on (`wait4(2)`'s own semantics, not `RUSAGE_SELF`).
/// Spawn a shell whose *child* (not the shell itself) burns real CPU, make
/// sure the shell is not the last command (so it can't tail-exec away and
/// must genuinely fork+wait), then confirm that CPU shows up when we wait4
/// the shell.
#[cfg(unix)]
#[test]
fn wait4_includes_waited_descendant_cpu() {
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg("dd if=/dev/zero of=/dev/null bs=1024 count=200000 2>/dev/null; true");
    let child = cmd.spawn().unwrap();
    let r = child.wait4().unwrap();

    assert!(r.status.success());
    assert!(r.rusage.utime + r.rusage.stime > Duration::from_millis(5));
}

#[cfg(unix)]
#[test]
fn try_wait4_pid_reaps_exited_child() {
    let child = true_command().spawn().unwrap();
    let pid = child.id();
    // wait4::try_wait4_pid reaps directly by pid, bypassing Child's own
    // bookkeeping; don't also call any Child wait method on it (Child has
    // no reaping Drop impl, so just letting it drop here is safe).
    drop(child);

    let r = loop {
        if let Some(r) = wait4::try_wait4_pid(pid).unwrap() {
            break r;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    assert!(r.status.success());
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
