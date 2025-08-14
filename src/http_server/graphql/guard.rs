use crate::http_server::controllers::ApiKey;
use async_graphql::*;

pub struct UserGuard;

impl Guard for UserGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if let Some(_) = ctx.data_opt::<ApiKey>() {
            // TODO: api key check
            // TODO: counting api request

            Ok(())
        } else {
            Err("Api key not found".into())
        }
    }
}
