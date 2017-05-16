use nfqueue;
use std::net::IpAddr;

struct Connection {
    pub process_name: String,
    pub to: IpAddr,
    pub verdict: nfqueue::Verdict
}

pub struct State {
    count: u32,
    connection_db: Vec<Connection>
}

impl State {
    pub fn new() -> State {
        State{ count:0, connection_db: vec!() }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn add_connection(&mut self, process_name: &str, to: IpAddr, verdict: nfqueue::Verdict) {
        self.connection_db.push(Connection{
            process_name: process_name.to_string(),
            to: to,
            verdict: verdict
        });
    }

    pub fn get_verdict(&self, process_name: &str, to: &IpAddr) -> Option<nfqueue::Verdict> {
        match self.connection_db.iter().find(|conn| conn.process_name == process_name && conn.to.eq(to)) {
            Some(ref conn) => { Some(conn.verdict.clone()) },
            None => None
        }
    }
}
