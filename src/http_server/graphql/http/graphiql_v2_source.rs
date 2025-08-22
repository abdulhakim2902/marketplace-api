use std::collections::HashMap;

use handlebars::Handlebars;
use serde::Serialize;

/// Indicates whether the user agent should send or receive user credentials
/// (cookies, basic http auth, etc.) from the other domain in the case of
/// cross-origin requests.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Credentials {
    /// Send user credentials if the URL is on the same origin as the calling
    /// script. This is the default value.
    #[default]
    SameOrigin,
    /// Always send user credentials, even for cross-origin calls.
    Include,
    /// Never send or receive user credentials.
    Omit,
}

#[derive(Serialize)]
struct GraphiQLVersion<'a>(&'a str);

impl Default for GraphiQLVersion<'_> {
    fn default() -> Self {
        Self("4")
    }
}

#[derive(Default, Serialize)]
pub struct GraphiQLSource<'a> {
    endpoint: &'a str,
    subscription_endpoint: Option<&'a str>,
    version: GraphiQLVersion<'a>,
    headers: Option<HashMap<&'a str, &'a str>>,
    ws_connection_params: Option<HashMap<&'a str, &'a str>>,
    title: Option<&'a str>,
    credentials: Credentials,
}

impl<'a> GraphiQLSource<'a> {
    /// Creates a builder for constructing a GraphiQL (v2) HTML page.
    pub fn build() -> GraphiQLSource<'a> {
        Default::default()
    }

    /// Sets the endpoint of the server GraphiQL will connect to.
    #[must_use]
    pub fn endpoint(self, endpoint: &'a str) -> GraphiQLSource<'a> {
        GraphiQLSource { endpoint, ..self }
    }

    /// Sets the subscription endpoint of the server GraphiQL will connect to.
    pub fn subscription_endpoint(self, endpoint: &'a str) -> GraphiQLSource<'a> {
        GraphiQLSource {
            subscription_endpoint: Some(endpoint),
            ..self
        }
    }

    /// Sets a header to be sent with requests GraphiQL will send.
    pub fn header(self, name: &'a str, value: &'a str) -> GraphiQLSource<'a> {
        let mut headers = self.headers.unwrap_or_default();
        headers.insert(name, value);
        GraphiQLSource {
            headers: Some(headers),
            ..self
        }
    }

    /// Sets the version of GraphiQL to be fetched.
    pub fn version(self, value: &'a str) -> GraphiQLSource<'a> {
        GraphiQLSource {
            version: GraphiQLVersion(value),
            ..self
        }
    }

    /// Sets a WS connection param to be sent during GraphiQL WS connections.
    pub fn ws_connection_param(self, name: &'a str, value: &'a str) -> GraphiQLSource<'a> {
        let mut ws_connection_params = self.ws_connection_params.unwrap_or_default();
        ws_connection_params.insert(name, value);
        GraphiQLSource {
            ws_connection_params: Some(ws_connection_params),
            ..self
        }
    }

    /// Sets the html document title.
    pub fn title(self, title: &'a str) -> GraphiQLSource<'a> {
        GraphiQLSource {
            title: Some(title),
            ..self
        }
    }

    /// Sets credentials option for the fetch requests.
    pub fn credentials(self, credentials: Credentials) -> GraphiQLSource<'a> {
        GraphiQLSource {
            credentials,
            ..self
        }
    }

    /// Returns a GraphiQL (v2) HTML page.
    pub fn finish(self) -> String {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string(
                "graphiql_v2_source",
                include_str!("./graphiql_v2_source.hbs"),
            )
            .expect("Failed to register template");

        handlebars
            .render("graphiql_v2_source", &self)
            .expect("Failed to render template")
    }
}
