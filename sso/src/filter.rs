use crate::helpers::base64_decode_to_string;
use anyhow::Result;
use headless_chrome::protocol::cdp::Network::events::ResponseReceivedEventParams;
use headless_chrome::protocol::cdp::Network::{GetResponseBodyReturnObject, ResourceType};
use tracing::error;

pub fn handler(
    params: ResponseReceivedEventParams,
    get_response_body: &dyn Fn() -> Result<GetResponseBodyReturnObject>,
    filters: &Filters,
) {
    //if !filters.pass(&params.response.url, &params.Type) {
    //    return;
    //}

    if let Ok(body) = get_response_body() {
        if body.base_64_encoded {
            error!(
                "URL: {} | {}",
                params.response.url,
                base64_decode_to_string(&body.body).unwrap_or("Decode failed".to_string())
            );
        } else {
            error!("URL: {} | {}", params.response.url, body.body);
        }
    } else {
        error!("Couldn't read response body for {}", params.response.url,);
    }
}

pub struct Filters {
    pub urls: Vec<String>,
}

impl Filters {
    fn pass(&self, url: &str, resource_type: &ResourceType) -> bool {
        // let url_matched = self.urls.iter().any(|x| url.contains(x));

        let res_type = match resource_type {
            ResourceType::Document => true,
            ResourceType::Stylesheet => false,
            ResourceType::Image => false,
            ResourceType::Media => false,
            ResourceType::Font => false,
            ResourceType::Script => false,
            ResourceType::TextTrack => false,
            ResourceType::Xhr => false,
            ResourceType::Fetch => false,
            ResourceType::EventSource => false,
            ResourceType::WebSocket => false,
            ResourceType::Manifest => false,
            ResourceType::SignedExchange => false,
            ResourceType::Ping => false,
            ResourceType::CspViolationReport => false,
            ResourceType::Preflight => false,
            ResourceType::Other => false,
        };

        // url_matched && res_type
        res_type
    }
}
