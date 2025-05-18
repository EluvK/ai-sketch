use salvo::{Depot, Response, Router, handler, writing::Text};

use crate::app_data::AppDataRef;

pub fn create_router() -> Router {
    Router::new().get(index_handler)
}

#[handler]
async fn index_handler(res: &mut Response) {
    res.render(Text::Plain("Data Monitor is running."));
}

#[handler]
async fn get_data(depot: &mut Depot, res: &mut Response) -> anyhow::Result<()> {
    let state = depot.obtain::<AppDataRef>().unwrap();

    // let data = state.data.lock().unwrap();
    // let joined = data.iter().cloned().collect::<Vec<_>>().join("\n");
    // res.render(Text::Plain(joined));
    Ok(())
}
