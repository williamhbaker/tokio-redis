use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        tx: oneshot::Sender<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        tx: oneshot::Sender<()>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key, tx } => {
                    let res = client.get(&key).await;
                    let _ = tx.send(res.unwrap());
                }
                Set { key, val, tx } => {
                    let res = client.set(&key, val).await.unwrap();
                    let _ = tx.send(res);
                }
            }
        }
    });

    let tx2 = tx.clone();
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        tx.send(Command::Set {
            key: "Hello".to_string(),
            val: "World".into(),
            tx: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        tx2.send(Command::Get {
            key: "Hello".to_string(),
            tx: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
