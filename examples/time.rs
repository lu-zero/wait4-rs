use std::env::args_os;
use std::process::Command;

use human_format::*;
use wait4::Wait4;

fn main() {
    let command: Vec<_> = args_os().collect();
    let mut b = Formatter::new();

    b.with_units("B")
        .with_scales(Scales::Binary())
        .with_separator("");

    let cmd = command.get(1).expect("Provide a command");
    let (_, args) = command.split_at(2);

    let now = std::time::Instant::now();
    let mut child = Command::new(cmd).args(args).spawn().unwrap();

    let r = child.wait4().unwrap();
    let elapsed = now.elapsed();

    println!("User time: {:.3?}", r.rusage.utime);
    println!("System time: {:.3?}", r.rusage.stime);
    println!("Elapsed: {:.3?}", elapsed);
    println!(
        "Maximum resident set size: {}",
        b.format(r.rusage.maxrss as f64)
    );
}
