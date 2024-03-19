#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::must_use_candidate)]

pub mod output;
pub mod shareable_content;
pub mod stream;
pub mod utils;

#[cfg(test)]
mod test {
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

    fn get_pid_name(pid: u32) -> String {
        // ps -p 66393 -o comm=
        let ret = process::Command::new("ps")
            .arg("-p")
            .arg(format!("{}", pid))
            .arg("-o")
            .arg("comm=")
            .output();

        String::from_utf8_lossy(&ret.unwrap().stdout).to_string()
    }
    #[test]
    fn test_process() {
        let pid = process::id();
        let pids = get_parent_pid(pid);
        let mut s = String::new();
        for p in pids {
            s += &get_pid_name(p);
        }
        assert_eq!(s, "aaa");
    }
}
