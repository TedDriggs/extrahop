use extrahop::{ApiResponse, Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct Username(String);

impl Username {
    fn new(name: impl Into<String>) -> Self {
        Username(name.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Dashboard {
    pub id: usize,
    pub owner: Option<Username>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DashboardTransfer {
    owner: Username,
}

impl DashboardTransfer {
    fn new(to: Username) -> Self {
        DashboardTransfer { owner: to }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new_appliance("sample-vm", "YOUR KEY".into(), Default::default()).await?;
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
                .patch(&format!("v1/dashboards/{}", dashboard.id))?
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
