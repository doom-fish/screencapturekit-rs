use std::process;

fn get_parent_pid(pid: u32) -> Vec<u32> {
    let mut pids: Vec<u32> = Vec::new();
    // ps -o ppid=66393
    let ret = process::Command::new("ps")
        .arg("-o")
        .arg(format!("ppid={}", pid))
        .output();

    if ret.is_err() {
        return pids;
    }

    let output = String::from_utf8_lossy(&ret.unwrap().stdout).to_string();
    for pid in output.split("\n") {
        match pid.parse::<u32>() {
            Ok(p) => pids.push(p),
            Err(_) => break,
        }
    }
    pids
}

fn get_pid_name(pid: u32) {
    // ps -p 66393 -o comm=
    let ret = process::Command::new("ps")
        .arg("-p")
        .arg(format!("{}", pid))
        .arg("-o")
        .arg("comm=")
        .output();

    let name = String::from_utf8_lossy(&ret.unwrap().stdout).to_string();

    println!("{:?}", name);
}

fn main() {
    let pid = process::id();
    let pids = get_parent_pid(pid);

    for p in pids {
        get_pid_name(p);
    }
}
