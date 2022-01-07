use extrahop::{ApiResponse, Client};
use filter_ast::Expr;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Device {
    display_name: String,
}

/// This function is agnostic towards which backend it connects to, so it accepts `Client`.
///
/// If we're using an API that is only available from appliances, we would instead take `ApplianceClient`
/// to signal that to callers.
async fn search_devices(client: &Client) -> anyhow::Result<Vec<Device>> {
    client
        .post("v1/devices/search")?
        .json(
            &(Expr::new_clause("software", "!=", "windows")
                & Expr::new_clause("ipaddr", "=", "123.156.189.0/24")),
        )
        .send()
        .await?
        .validate_and_read::<Vec<Device>>()
        .await
        .map_err(anyhow::Error::from)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new_saas("example.cloud.extrahop.com", "123".into(), "456".into()).await?;
    let devices = search_devices(&client).await?;
    for device in devices {
        println!("{}", device.display_name);
    }
    Ok(())
}
