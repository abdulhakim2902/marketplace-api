use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MarketplaceSchema {
    pub name: String,
    pub contract_address: String,
}

#[async_graphql::Object]
impl MarketplaceSchema {
    #[graphql(name = "contract_address")]
    async fn contract_address(&self) -> &str {
        &self.contract_address
    }

    async fn name(&self) -> &str {
        &self.name
    }
}
