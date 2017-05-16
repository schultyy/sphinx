extern crate nfqueue;
extern crate libc;
extern crate pnet;
extern crate regex;
#[macro_use] extern crate text_io;

mod state;
mod process_mon;

use std::process;
use state::State;
use std::net::IpAddr;

use pnet::packet::ipv4::Ipv4Packet;

fn print_err(message: &str) {
    println!("[ERR] {}", message);
}

fn queue_callback(msg: &nfqueue::Message, state: &mut State) {
    state.increment();
    let processes = process_mon::active_connections();

    if let Some(header) = Ipv4Packet::new(msg.get_payload()) {
        let source = IpAddr::V4(header.get_source());
        let destination = IpAddr::V4(header.get_destination());
        let known_connection = processes
            .iter()
            .filter(|process| process.matches(&source, &destination)).next();

        if known_connection.is_some() {
            let conn = known_connection.unwrap();

            if let Some(verdict) = state.get_verdict(&conn.process, &destination) {
                msg.set_verdict(verdict);
            } else {
                println!("Intercepted(Process: {} - {}) {} -> {}", conn.process, conn.pid, source, destination);
                print!("Accept this connection? <y/n> ");
                let answer :String = read!();
                if answer == "y" {
                    msg.set_verdict(nfqueue::Verdict::Accept);
                    state.add_connection(&conn.process, destination, nfqueue::Verdict::Accept);
                } else {
                    msg.set_verdict(nfqueue::Verdict::Drop);
                    state.add_connection(&conn.process, destination, nfqueue::Verdict::Drop);
                }
            }
        }
    }
}

fn main() {
    println!("Sphinx - Keeps watch");
    let mut q = nfqueue::Queue::new(State::new());
    q.open();
    q.unbind(libc::AF_INET);

    let bind_result = q.bind(libc::AF_INET);
    if bind_result != 0 {
        print_err("Creating bind failed. Do you run this as root? Exiting");
        process::exit(1)
    }

    q.create_queue(0, queue_callback);
    q.set_mode(nfqueue::CopyMode::CopyPacket, 0xffff);

    q.run_loop();
    q.close();
}
