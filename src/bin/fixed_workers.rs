use core::time;

struct Processed {
    value: String,
}

struct Work {
    value: String,
}

async fn do_work(work: Work) -> Processed {
    tokio::time::sleep(time::Duration::from_millis(500)).await;

    Processed {
        value: format!("{}_processed", work.value),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker_count = 5;

    let (tx_work, rx_work): (async_channel::Sender<Work>, async_channel::Receiver<Work>) =
        async_channel::bounded(1);

    let (tx_processed, rx_processed): (
        async_channel::Sender<Processed>,
        async_channel::Receiver<Processed>,
    ) = async_channel::bounded(1);

    let mut workers = Vec::new();

    for _ in 0..worker_count {
        workers.push(tokio::spawn(worker(rx_work.clone(), tx_processed.clone())));
    }

    let consumer = tokio::spawn(consumer(rx_processed));

    for idx in 0..20 {
        tx_work
            .send(Work {
                value: format!("work_{}", idx),
            })
            .await?;
    }
    drop(tx_work);

    // Wait for all workers to be done.
    futures::future::try_join_all(workers).await?;

    // Close the tx_processed channel since workers will not send on that anymore. All of the
    // workers will have exited at this point and dropped their clones of this, so dropping this
    // last sender closes the channel.
    drop(tx_processed);

    // Wait for the consumer to be done.
    consumer.await?;

    Ok(())
}

async fn worker(input: async_channel::Receiver<Work>, output: async_channel::Sender<Processed>) {
    loop {
        match input.recv().await {
            Ok(work) => {
                if output.send(do_work(work).await).await.is_err() {
                    return;
                };
            }
            Err(e) => {
                println!("shutting down worker: {}", e);
                return;
            }
        }
    }
}

async fn consumer(input: async_channel::Receiver<Processed>) {
    loop {
        match input.recv().await {
            Ok(processed) => println!("{}", processed.value),
            Err(e) => {
                println!("shutting down consumer: {}", e);
                return;
            }
        }
    }
}
