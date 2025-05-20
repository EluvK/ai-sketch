use salvo::{Response, Router, handler, writing::Text};

mod user;

pub fn create_router() -> Router {
    Router::with_path("meter")
        .get(index_handler)
        .push(Router::with_path("user").push(user::router()))
}

#[handler]
async fn index_handler(res: &mut Response) {
    res.render(Text::Plain("Data Monitor is running."));
}
