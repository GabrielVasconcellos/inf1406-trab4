use mosquitto_client::Mosquitto;
use serde_json::json;
use std::process::Command;
use std::time::{Duration, Instant};
use std::{env, thread};

fn monitor(n: usize) {
    let m = Mosquitto::new("monitor");
    let topic = "inf1406-monitor";

    m.connect("localhost", 1883);
    m.subscribe(topic, 1);

    println!("Monitor inicializado");

    let mut mc = m.callbacks(Vec::new());

    let mut timeouts: Vec<Instant> = Vec::with_capacity(n);

    for mut t in timeouts {
        t = Instant::now();
    }

    loop {
        mc.on_message(|data, msg| {
            let heartbeat = serde_json::Value::from(msg.text());

            timeouts[i32::from(heartbeat["idServ"].to_string())] = Instant::now();
        });

        for (i, t) in timeouts.iter().enumerate() {
            if t.duration_since(Instant::now()) > 1000 {
                let req = json!({
                   "tipomsg": "falhaserv",
                    "idServ": i.to_string(),
                    "vistoem": t
                });
                m.publish("inf1406-reqs", req.to_string().as_bytes(), 1, false);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let server_cnt = args[1].parse::<usize>().unwrap();
    let client_cnt = args[2].parse::<usize>().unwrap();

    println!("Inicializando os servidores e clients");
    println!("-------------------------------------------");

    let mut servers = Vec::with_capacity(server_cnt);
    for i in 0..server_cnt {
        let child = Command::new("src/server")
            .arg(i.to_string())
            .arg(server_cnt.to_string())
            .spawn()
            .expect("failed to execute child");

        servers.push(child);
    }

    let mut clients = Vec::with_capacity(client_cnt);
    for i in 0..client_cnt {
        let mut req_type = "query";
        if i % 2 == 0 {
            req_type = "insert";
        }
        let child = Command::new("src/client")
            .arg(i.to_string())
            .arg(req_type)
            .spawn()
            .expect("failed to execute child");

        clients.push(child);
    }

    thread::spawn(move || monitor(server_cnt));

    for i in 0..client_cnt {
        clients[i].wait().unwrap();
    }

    for i in 0..server_cnt {
        servers[i].wait().unwrap();
    }
}
