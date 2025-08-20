use crate::http_server::controllers::ApiKey;
use async_graphql::*;
use prefixed_api_key::{PrefixedApiKey, PrefixedApiKeyController};

pub struct UserGuard;

impl Guard for UserGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if let Some(api_key) = ctx.data_opt::<ApiKey>() {
            if !api_key.active {
                return Err("Account is inactive".into());
            }

            let prefixed_api_key = PrefixedApiKey::from_string(&api_key.key)?;
            let controller = PrefixedApiKeyController::configure()
                .prefix("ucc".to_owned())
                .seam_defaults()
                .finalize()?;

            let is_authorize = controller.check_hash(&prefixed_api_key, &api_key.token_hash);

            if is_authorize {
                Ok(())
            } else {
                Err("Api key is invalid".into())
            }
        } else {
            Err("Api key not found".into())
        }
    }
}
