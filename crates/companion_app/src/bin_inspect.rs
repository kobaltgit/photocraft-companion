use photocraft_bridge::PhotoCraftClient;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = PhotoCraftClient::new(18492, None);
    client.connect().await?;
    let inspect = client.execute_command("document.inspect", serde_json::json!({})).await?;
    println!("Doc width: {}, height: {}", inspect.get("width").unwrap(), inspect.get("height").unwrap());
    let layers = inspect.get("layers").unwrap().as_array().unwrap();
    for l in layers {
        println!("ID: {:<3} Name: {:<20} Bounds: {:?}", l.get("id").unwrap(), l.get("name").unwrap().as_str().unwrap(), l.get("bounds").unwrap());
    }
    Ok(())
}
