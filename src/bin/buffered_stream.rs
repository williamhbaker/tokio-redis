use futures::StreamExt;

struct Processed {
    value: String,
}

#[derive(Debug)]
struct Work {
    value: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx_work, rx_work) = tokio::sync::mpsc::channel(100);

    let consumer = tokio::spawn(consumer(rx_work));

    for idx in 0..20 {
        tx_work
            .send(Work {
                value: format!("work_{}", idx),
            })
            .await?;
    }
    drop(tx_work);

    consumer.await?;

    Ok(())
}

async fn consumer(mut incoming: tokio::sync::mpsc::Receiver<Work>) {
    let stream = async_stream::stream! {
        while let Some(item) = incoming.recv().await {
            yield do_work(item);
        }
    };

    let queue = stream.buffer_unordered(5);
    futures::pin_mut!(queue);

    while let Some(result) = queue.next().await {
        println!("{}_processed", result.value);
    }

    ()
}

async fn do_work(work: Work) -> Processed {
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    Processed {
        value: format!("{}_processed", work.value),
    }
}
