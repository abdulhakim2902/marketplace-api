use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MarketplaceSchema {
    pub id: Uuid,
    pub name: String,
    pub contract_address: String,
}

#[async_graphql::Object]
impl MarketplaceSchema {
    async fn id(&self) -> String {
        self.id.to_string()
    }
    
    #[graphql(name = "contract_address")]
    async fn contract_address(&self) -> &str {
        &self.contract_address
    }

    async fn name(&self) -> &str {
        &self.name
    }
}
