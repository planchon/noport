use http_body_util::combinators::BoxBody;
use hyper::{Response, body::Bytes};

use noport_lib::store::Store;

pub async fn handle_request(
    req: hyper::Request<hyper::body::Incoming>,
    store: Store,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    println!("Handling request {:?}", req);

    Err(hyper::Error::from())
}
