use anyhow::Result;
use comcigan_rs::{init, search_school, view};

#[cfg(feature = "hyper")]
use comcigan_rs::client::HyperClient;

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::SimpleLogger::new().init().unwrap();
    use std::time::Instant;
    let now = Instant::now();
    let client = HyperClient::new();

    let keys = init(&client).await?;

    let schools = search_school(&client, "세종과학고등학교", &keys).await?;
    let school = view(&client, &schools[0], &keys).await?;

    // 1학년 1반 금요일 4교시
    let day = school.grade(1).class(1).day(5);

    for period in day.list_periods() {
        println!("{}\n", period);
    }


    let then = Instant::now();

    println!("Time elapsed: {}", then.duration_since(now).as_millis());
    Ok(())
}