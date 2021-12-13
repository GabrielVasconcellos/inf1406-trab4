use mosquitto_client::Mosquitto;
use serde_json::json;
use std::cmp::max;
use std::env;
use std::string::String;

fn main() {
    let args: Vec<String> = env::args().collect();
    let id = args[1].parse::<i32>().unwrap();
    let req_type = args[2].to_string();
    let value = format!("valor{}", id);
    let req_id = format!("c{}_req", id);
    let m = Mosquitto::new(&format!("c{}", id));

    match m.connect("localhost", 1883) {
        Ok(val) => val,
        Err(e) => println!("Client {}: {}", id, e),
    };

    println!("Client {} inicializado", id);

    let req = json!({
        "tipomsg": req_type,
        "chave": (id % 3).to_string(),
        "topico-resp": req_id,
        "idpedido": req_id,
        "novovalor": value
    });

    m.subscribe(&req["topico-resp"].to_string(), 1);

    let mut mc = m.callbacks(Vec::new());

    m.publish("inf1406-reqs", req.to_string().as_bytes(), 1, false)
        .unwrap();

    mc.on_message(|data: &mut Vec<String>, msg| {
        if req_type == "insert" {
            println!(
                "Pedido {}: {} na chave {}",
                req_id,
                msg.text(),
                req["chave"]
            );
        } else if msg.text() == "NA" {
            println!(
                "Pedido {}: valor indisponível na consulta da chave {}",
                req["idpedido"], req["chave"]
            );
        } else {
            println!(
                "Pedido {}: resultado da consulta da chave {} é {} ",
                req["idpedido"],
                req["chave"],
                msg.text()
            );
        }

        data.push(String::from(msg.text()));
    });

    m.loop_until_disconnect(200);
}
