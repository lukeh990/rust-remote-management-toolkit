use warp::Filter;
use chrono::prelude::*;
use rrss_lib::Bug;

#[tokio::main]
async fn main() {
    let bug = warp::post()
        .and(warp::path("bug"))
        .and(warp::body::json())
        .map(|body: Bug| {
            println!("{} | {1} | {2}", body.machine, Utc::now(), body.body);
            warp::reply()
        });

    warp::serve(bug).run(([0, 0, 0, 0], 9000)).await
}