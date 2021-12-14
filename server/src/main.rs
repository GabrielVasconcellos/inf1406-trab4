use mosquitto_client::Mosquitto;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::sync::mpsc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn get_hash(key: &String, n: i32) -> i32 {
    let mut sum: i32 = 0;
    let byte_slice = key.as_bytes();

    for el in byte_slice.iter() {
        sum += i32::from(*el);
    }
    sum % n
}

fn handle_request(
    mt: Mosquitto,
    req: Value,
    id: i32,
    tx: Sender<(serde_json::Value, bool, Sender<(String, serde_json::Value)>)>,
    hash: i32,
    is_sub: bool,
) {
    let (ty, ry): (
        Sender<(String, serde_json::Value)>,
        Receiver<(String, serde_json::Value)>,
    ) = mpsc::channel();

    let mut is_target_server = false;

    if hash == id || (is_sub && hash == (id + 1) % 4) {
        is_target_server = true;
    }

    tx.send((req.clone(), is_target_server, ty)).unwrap();
    thread::spawn(move || {
        for (str, val) in ry {
            if is_target_server {
                mt.publish(&val["topico-resp"].to_string(), str.as_bytes(), 1, false)
                    .unwrap();
            }
        }
    });
}

fn main() -> Result<(), mosquitto_client::Error> {
    let args: Vec<String> = env::args().collect();
    let id = args[1].parse::<i32>().unwrap();
    let n = args[2].parse::<i32>().unwrap();
    let heartbeat_period = args[3].parse::<u64>().unwrap();

    println!("Server {} inicializado", id);

    let (tx, rx): (
        Sender<(serde_json::Value, bool, Sender<(String, serde_json::Value)>)>,
        Receiver<(serde_json::Value, bool, Sender<(String, serde_json::Value)>)>,
    ) = channel();

    thread::spawn(move || {
        let mut hashmap = HashMap::new();

        for (v, is_query_server, ty) in rx {
            if v["tipomsg"] == "insert" {
                let value = match hashmap.insert(v["chave"].to_string(), v["novovalor"].to_string())
                {
                    Some(val) => format!("valor {} atualizado para {}", val, v["novovalor"]),
                    None => format!("valor {} inserido", v["novovalor"]),
                };
                ty.send((value, v)).unwrap();
            } else if is_query_server {
                let hm = hashmap.clone();
                let value = match hm.get(&v["chave"].to_string()) {
                    Some(val) => val,
                    None => "NA",
                };
                ty.send((value.to_string(), v)).unwrap();
            }
        }
    });

    let (ts, rs): (
        Sender<(String, bool, Option<Sender<bool>>)>,
        Receiver<(String, bool, Option<Sender<bool>>)>,
    ) = channel();

    thread::spawn(move || {
        let mut is_sub = false;

        for (str, b, t) in rs {
            if str == "update" {
                is_sub = b;
            } else {
                t.unwrap().send(is_sub);
            }
        }
    });

    let m = mosquitto_client::Mosquitto::new(&format!("s{}", id));

    m.connect("localhost", 1883)?;
    m.subscribe("inf1406-reqs", 1)?;

    let mut mc = m.callbacks(Vec::new());

    mc.on_message(|data: &mut Vec<Vec<(Instant, String)>>, msg| {
        let tx_clone = tx.clone();
        let mt = m.clone();
        let str = msg.text().to_string();
        let req: Value = serde_json::from_str(&str).unwrap();
        let hash = get_hash(&req["chave"].to_string(), n) as usize;

        if data.is_empty() {
            for _i in 0..n {
                data.push(Vec::with_capacity(n as usize));
            }
        } else {
            let log = data[hash].clone();

            let mut server_id = -1;
            if req["tipomsg"] == "falhaserv" {
                server_id = (req["idServ"].to_string().parse::<i32>().unwrap() + 1) % 4;
                ts.send(("update".to_string(), true, None));
            }

            if server_id == id {
                for (_, l) in log {
                    handle_request(
                        mt.clone(),
                        serde_json::from_str(&l).unwrap(),
                        id,
                        tx_clone.clone(),
                        hash as i32,
                        false,
                    );
                }
            }
        }

        let (tsub, rsub): (Sender<bool>, Receiver<bool>) = channel();
        ts.send(("valor".to_string(), false, Option::from(tsub)));

        let is_sub = rsub.recv().unwrap();
        handle_request(mt, req, id, tx_clone, hash as i32, is_sub);

        let last_time = match data[hash].last() {
            Some((time, _req)) => *time,
            None => Instant::now(),
        };
        let first_time = match data[hash].first() {
            Some((time, _req)) => *time,
            None => last_time,
        };
        if last_time.duration_since(first_time) > Duration::from_millis(20 * heartbeat_period) {
            data[hash].clear();
            data[hash].push((Instant::now(), "Log Start".to_string()));
        }
        data[hash].push((Instant::now(), str));
    });

    let mt = m.clone();
    thread::spawn(move || loop {
        let heartbeat = json!({ "idServ": id.to_string() });
        mt.publish(
            "inf1406-monitor",
            heartbeat.to_string().as_bytes(),
            1,
            false,
        );
        sleep(Duration::from_millis(heartbeat_period));
    });

    m.loop_until_disconnect(200);

    Ok(())
}
