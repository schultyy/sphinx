use std::process::Command;
use regex::Regex;


fn parse(regex: &str, process_line: &str) -> Option<String> {
    let regex = Regex::new(regex).unwrap();
    let capture = regex.captures_iter(&process_line).next();
    if capture.is_none() {
        return None
    }
    capture
        .unwrap()
        .get(1)
        .map_or(None, |s| Some(s.as_str().into()))
}

// fn parse_process(process_line: &str) -> String {
//     parse(r"users:\(\(\\"(\\w+)\"", process_line)
// }

fn parse_pid(process_line: &str) -> Option<String> {
    parse(r"pid=(\d+)", process_line)
}

fn parse_ip_addresses(process_line: &str) -> Option<(String, String)> {
    let regex = Regex::new(r"(\d{1,3}.\d{1,3}.\d{1,3}.\d{1,3})+:\d+").unwrap();
    let mut captures = regex.captures_iter(&process_line);
    let from_capture = captures.next();
    let to_capture = captures.next();
    if from_capture.is_none() || to_capture.is_none() {
        return None
    }

    let from = from_capture
                .unwrap()
                .get(1)
                .map_or("".into(), |s| s.as_str().into());
    let to = to_capture
                .unwrap()
                .get(1)
                .map_or("".into(), |s| s.as_str().into());
    Some((from, to))
}

pub struct Process {
    pub pid: String,
    pub from: String,
    pub to: String
}

impl Process {
    pub fn new(process_line: &str) -> Option<Process> {
        let (from, to) = match parse_ip_addresses(process_line) {
            Some((f, t)) => (f, t),
            None => ("".into(), "".into())
        };

        let pid_or_none = parse_pid(process_line);

        if pid_or_none.is_none() {
            None
        } else {
            Some(Process {
                pid: pid_or_none.unwrap(),
                from: from,
                to: to
            })
        }
    }
}

fn get_process_table() -> String {
    let socket_list = Command::new("ss")
                                .arg("-nap")
                                .arg("-A")
                                .arg("inet")
                                .output()
                                .expect("Failed to execute `ss`. Is the program installed on your system?");
    String::from_utf8(socket_list.stdout).unwrap()
}


pub fn active_connections() -> Vec<Process> {
    get_process_table()
        .split("\n")
        .map(|s| Process::new(s))
        .filter(|p| p.is_some())
        .map(|p| p.unwrap())
        .collect::<Vec<Process>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT : &'static str = "tcp   ESTAB      0      0                                                              192.168.178.36:57222                                                                      192.30.253.125:443                 users:((\"firefox\",pid=2704,fd=98))";

    #[test]
    fn it_extracts_pid() {
        let expected_pid = "2704";
        let actual_pid = parse_pid(INPUT);
        assert_eq!(expected_pid, actual_pid);
    }

    // #[test]
    // #[ignore]
    // fn it_extracts_process() {
    //     let expected_process = "firefox";
    //     let actual_process = parse_process(INPUT);
    //     assert_eq!(expected_process, actual_process);
    // }

    #[test]
    fn it_parses_ip_addresses() {
        let (actual_from, actual_to) = parse_ip_addresses(INPUT).unwrap();
        assert_eq!("192.168.178.36", actual_from);
        assert_eq!("192.30.253.125", actual_to);
    }

    #[test]
    fn it_returns_a_populated_struct() {
        let process = Process::new(INPUT).unwrap();
        assert_eq!("192.168.178.36", process.from);
        assert_eq!("192.30.253.125", process.to);
        assert_eq!("2704", process.pid);
    }

    #[test]
    fn it_bails_out_if_ip_address_is_ipv6() {
        let input_ipv6 = "tcp   LISTEN     0      128                                                                                                                             :::34071                                                                                                                                       :::*                   users:((\"code\",pid=3907,fd=41))";
        let result = parse_ip_addresses(input_ipv6);
        assert_eq!(None, result);
    }
}
