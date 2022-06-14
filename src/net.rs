// use torut;

use std::{
    io::Read,
    net::{Shutdown, TcpListener, TcpStream},
    thread,
};

pub fn listen(address: Option<String>) {
    let listener = match address {
        Some(addr) => TcpListener::bind(addr),
        None => TcpListener::bind("0.0.0.0:5000"),
    }
    .unwrap();

    let _ = listener.set_nonblocking(true);

    log::info!(
        "Forage listening on port {}",
        listener.local_addr().unwrap().port()
    );
    for stream in listener.incoming().flatten() {
        log::info!(
            "Connection recieved with peer {}",
            stream.peer_addr().unwrap()
        );
        let _ = thread::spawn(move || handle_client(stream));
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut data: Vec<u8> = Vec::new();
    while match stream.read(&mut data) {
        Ok(_) => true,
        Err(_) => {
            log::error!(
                "error occured, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
    println!("{:?}", data);
}
