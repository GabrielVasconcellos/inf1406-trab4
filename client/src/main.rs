use mosquitto_client::Mosquitto;
use serde_json::json;
use std::env;

fn main() -> Result<(), mosquitto_client::Error> {
    let args: Vec<String> = env::args().collect();
    let id = args[1].parse::<i32>().unwrap();
    let n = args[2].parse::<i32>().unwrap();
    let tipomsg = args[3].to_string();

    let m = Mosquitto::new(&format!("c{}", id));

    match m.connect("localhost", 1883) {
        Ok(val) => val,
        Err(e) => println!("Client {}: {}", id, e)
    };

    println!("Client {} inicializado", id);

    let chave = "chave";
    let topico_resp = "resp";
    let idpedido = "id";
    let novovalor = if tipomsg == "insert" { "Teste" } else { "" };

    let req = json!({
        "tipomsg": tipomsg,
        "chave": chave,
        "topico-resp": topico_resp,
        "idpedido": idpedido,
        "novovalor": novovalor
    });

    m.subscribe(&req["topico-resp"].to_string(), 1);

    let mut mc = m.callbacks(Vec::new());

    m.publish("inf1406-reqs", req.to_string().as_bytes(), 1, false).unwrap();

    mc.on_message(|data: &mut Vec<String>, msg| {
        if tipomsg == "insert" {
            println!("Pedido {}: {}", idpedido, msg.text());
        } else if msg.text() == "NA" {
            println!(
                "Pedido {}: valor indisponível na consulta da chave {}",
                idpedido, chave
            );
        } else {
            println!(
                "Pedido {}: resultado da consulta da chave {} é {} ",
                idpedido,
                chave,
                msg.text()
            );
        }
    });

    m.loop_until_disconnect(200)?;

    Ok(())
}


