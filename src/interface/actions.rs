/// Request processing module
///
use super::proto::{Request, Response};

pub fn process_request(request: Request) -> Response {

    // todo implement

    Response::Location {
        msg: "netloc to ground control".to_string(),
    }
}
