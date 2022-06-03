use std::time::{Duration, Instant};
use tokio_udt::UdtListener;

#[tokio::main]
async fn main() {
    let listener = UdtListener::bind("0.0.0.0:9000".parse().unwrap(), 10)
        .await
        .unwrap();

    println!("Waiting for connection...");

    let (addr, connection) = listener.accept().await.unwrap();

    println!("Accepted connection from {}", addr);

    let mut buffer = [0; 2048];
    let mut last = Instant::now();
    let mut count = 0;

    loop {
        let size = connection.recv(&mut buffer).await.unwrap();

        if size > 0 {
            count += 1;
        }

        if last.elapsed() > Duration::new(1, 0) {
            last = Instant::now();

            println!("Received {} packets (size is {})", count, size);
        }
    }
}
