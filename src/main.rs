#[tokio::main]
async fn main() {
    match urdobot_app::hello().await {
        Ok(data) => println!("Success: {data:#?}"),
        Err(e) => eprintln!("{e}"),
    }
}
