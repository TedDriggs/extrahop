use extrahop::{ApiResponse, Client, Username};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: usize,
    pub owner: Option<Username>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardTransfer {
    pub owner: Username,
}

impl DashboardTransfer {
    pub fn new(to: Username) -> Self {
        DashboardTransfer { owner: to }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::builder("sample-vm", "YOUR KEY")
        .dangerous_disable_cert_verification(true)
        .build()?;
    let dashboards = client
        .get("v1/dashboards")?
        .send()
        .await?
        .validate_and_read::<Vec<Dashboard>>()
        .await?;

    let from_user = Some(Username::new("kenp"));
    let patch = DashboardTransfer::new(Username::new("setup"));
    for dashboard in dashboards {
        if dashboard.owner == from_user {
            let transfer_result = client
                .patch(&format!("/dashboards/{}", dashboard.id))?
                .json(&patch)
                .send()
                .await?
                .validate_status()
                .await;

            match transfer_result {
                Err(e) => println!("Error: {}", e),
                Ok(..) => println!(
                    "Successfully transferred #{}, '{}'",
                    dashboard.id, dashboard.name
                ),
            };
        }
    }

    Ok(())
}
