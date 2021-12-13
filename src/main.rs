use std::process::{Command};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let server_cnt = args[1].parse::<usize>().unwrap();
    let client_cnt = args[2].parse::<usize>().unwrap();

    println!("Inicializando os servidores e clients");
    println!("-------------------------------------------");

    let mut servers = Vec::with_capacity(server_cnt);
    for i in 0..server_cnt
    {
        let child = Command::new("src/server")
            .arg(i.to_string())
            .arg(server_cnt.to_string())
            .spawn()
            .expect("failed to execute child");

        servers.push(child);
    }

    let mut clients = Vec::with_capacity(client_cnt);
    for i in 0..client_cnt
    {
        let child = Command::new("src/client")
            .arg(i.to_string())
            .arg("10")
            .arg("insert")
            .spawn()
            .expect("failed to execute child");

        clients.push(child);
    }

    for i in 0..client_cnt
    {
        clients[i].wait().unwrap();
    }

    for i in 0..server_cnt
    {
        servers[i].wait().unwrap();
    }
}
