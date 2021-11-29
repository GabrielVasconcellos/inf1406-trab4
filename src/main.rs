fn main() -> Result<(), mosquitto_client::Error> {
    use std::{thread,time};

    let m = mosquitto_client::Mosquitto::new("test");

    m.connect("localhost",1883)?;
    m.subscribe("bilbo/#",1)?;

    let mt = m.clone();
    thread::spawn(move || {
        let timeout = time::Duration::from_millis(500);
        for i in 0..5 {
            let msg = format!("hello #{}",i+1);
            mt.publish("bilbo/baggins",msg.as_bytes(), 1, false).unwrap();
            thread::sleep(timeout);
        }
        mt.disconnect().unwrap();
    });

    let mut mc = m.callbacks(Vec::new());
    mc.on_message(|data,msg| {
        data.push(msg.text().to_string());
    });

    m.loop_until_disconnect(200)?;
    assert_eq!(mc.data.len(),5);
    for msg in &mc.data {
        println!("{}", msg);
    }
    Ok(())
}
