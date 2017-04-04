extern crate extrahop;
extern crate reqwest;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use extrahop::{ApiKey, ApiResponse, Client, Error, ErrorKind, Username};

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

fn main() {
    let client = Client::new("ehd-vm", ApiKey::new("YOUR KEY".to_string()));
    let dashboards: Vec<Dashboard> =
        client.get("dashboards")
            .send()
            .validate_and_read()
            .unwrap();
    
    let from_user = Some(Username::new("kenp"));
    let patch = DashboardTransfer::new(Username::new("setup"));
    for dashboard in dashboards {
        if dashboard.owner == from_user {
            let transfer_result = client
                .patch(&format!("dashboards/{}", dashboard.id))
                .json(&patch)
                .send()
                .validate_status();
                
            match transfer_result {
                Err(Error(ErrorKind::Rest(rest), _)) => println!("Error: {}", rest),
                Err(e) => println!("Error: {}", e),
                Ok(..) => println!("Successfully transferred #{}, '{}'", dashboard.id, dashboard.name)
            };
        }
    }
}