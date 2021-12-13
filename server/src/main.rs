use mosquitto_client::Mosquitto;
use std::collections::HashMap;
use std::env;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use serde_json::Value;

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
    req: String,
    id: i32,
    tx: Sender<(serde_json::Value, bool, Sender<(String, serde_json::Value)>)>,
    n: i32,
) {
    let (ty, ry): (
        Sender<(String, serde_json::Value)>,
        Receiver<(String, serde_json::Value)>,
    ) = mpsc::channel();

    let v:Value = serde_json::from_str(&req).unwrap();

    let is_target_server = if id == get_hash(&v["chave"].to_string(), n) {
        true
    } else {
        false
    };

    tx.send((v.clone(), is_target_server, ty)).unwrap();
    for (str, val) in ry {
        if is_target_server {
            mt.publish(&val["topico-resp"].to_string(), str.as_bytes(), 1, false)
                .unwrap();
        }
    }
}

fn main() -> Result<(), mosquitto_client::Error> {
    let args: Vec<String> = env::args().collect();
    let id = args[1].parse::<i32>().unwrap();
    let n = args[2].parse::<i32>().unwrap();

    println!("Server {} inicializado", id);

    let (tx, rx): (
        Sender<(serde_json::Value, bool, Sender<(String, serde_json::Value)>)>,
        Receiver<(serde_json::Value, bool, Sender<(String, serde_json::Value)>)>,
    ) = mpsc::channel();

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
            } else if v["tipomsg"] == "query" && is_query_server {
                let hm = hashmap.clone();
                let value = match hm.get(&v["chave"].to_string()) {
                    Some(val) => val,
                    None => "NA",
                };
                ty.send((value.to_string(), v)).unwrap();
            }
        }
    });

    let m = mosquitto_client::Mosquitto::new(&format!("s{}", id));

    m.connect("localhost", 1883)?;
    m.subscribe("inf1406-reqs", 1)?;

    let mut mc = m.callbacks(Vec::new());
    mc.on_message(|data, msg| {
        let tx_clone = tx.clone();
        let mt = m.clone();
        let str = msg.text().to_string();

        thread::spawn(move || {
            handle_request(mt, str, id, tx_clone, n);
        });

        data.push(msg.text().to_string());
    });

    m.loop_until_disconnect(200)?;

    // println!("LOG:");
    // for msg in &mc.data {
    //     println!("{}", msg);
    // }
    Ok(())
}
