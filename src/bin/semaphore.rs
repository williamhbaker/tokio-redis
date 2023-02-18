use std::sync::Arc;

use tokio::{
    sync::{mpsc, Semaphore},
    time,
};

#[derive(Debug)]
struct Work {
    message: String,
}

#[derive(Debug)]
struct Result {
    message: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx_work, rx_work) = mpsc::channel(1);
    let (tx_result, mut rx_result) = mpsc::channel(1);

    tokio::spawn(worker(rx_work, tx_result));
    tokio::spawn(producer(tx_work));

    while let Some(result) = rx_result.recv().await {
        println!("got result: {}", result.message);
    }

    Ok(())
}

async fn producer(tx_work: mpsc::Sender<Work>) {
    for idx in 0..20 {
        tx_work
            .send(Work {
                message: format!("message_{}", idx),
            })
            .await
            .unwrap();
    }
}

async fn worker(
    mut rx_work: mpsc::Receiver<Work>,
    tx_result: mpsc::Sender<Result>,
) -> anyhow::Result<()> {
    let semaphore = Arc::new(Semaphore::new(5));

    while let Some(work) = rx_work.recv().await {
        let permit = semaphore.clone().acquire_owned().await?;
        let tx = tx_result.clone();
        tokio::spawn(async move {
            tx.send(do_work(work).await).await.unwrap();
            drop(permit)
        });
    }

    Ok(())
}

async fn do_work(work: Work) -> Result {
    time::sleep(std::time::Duration::from_millis(300)).await;

    Result {
        message: format!("{}_processed", work.message),
    }
}
