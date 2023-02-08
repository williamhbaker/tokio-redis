// Based on https://gendignoux.com/blog/2021/04/01/rust-async-streams-futures-part1.html

use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stream = get_items_stream();

    let buf = stream.buffer_unordered(5);

    futures::pin_mut!(buf);

    while let Some(got) = buf.next().await {
        println!("{got}");
    }

    return Ok(());
}

fn get_items_stream() -> impl futures::Stream<Item = impl futures::Future<Output = String>> {
    let (tx, rx) = std::sync::mpsc::sync_channel(32);

    std::thread::spawn(move || {
        let mut count = 0;
        loop {
            if tx.send(get_item(count)).is_err() {
                return;
            };
            count += 1;
        }
    });

    futures::stream::iter(rx.into_iter())
}

async fn get_item(count: usize) -> String {
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    format!("Item #{}", count)
}
