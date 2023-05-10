extern crate chrono;
extern crate env_logger;
extern crate iron;
extern crate logger;
extern crate router;
extern crate uuid;
extern crate serde_json;


mod models;
mod database;
mod handlers;

use database::Database;
use handlers::*;
use models::*;

use iron::prelude::Chain;
use iron::Iron;
use router::Router;
use uuid::Uuid;

struct LoggerBefore{}

impl LoggerBefore{
    fn new() -> LoggerBefore{
        LoggerBefore{}
    }
}

impl iron::middleware::BeforeMiddleware for LoggerBefore{
    fn before(&self, req: &mut iron::Request) -> iron::IronResult<()>{
        println!("----->logger_before: {:?}", req);
        Ok(())
    }
}

struct LoggerAfter{}

impl LoggerAfter{
    fn new() -> LoggerAfter{
        LoggerAfter{}
    }
}

impl iron::middleware::AfterMiddleware for LoggerAfter{
    fn after(&self, _: &mut iron::Request, res: iron::Response) -> iron::IronResult<iron::Response>{
        println!("------>logger_after: {:?}", res);
        Ok(res)
    }
}

fn main() {
    env_logger::init();

    let mut db = Database::new();
    let p = Post::new(
        "The First Post",
        "This is the first post in our API",
        "Tensor",
        chrono::offset::Utc::now(),
        Uuid::new_v4(),
    );
    db.add_post(p);

    let p2 = Post::new(
        "The next post is better",
        "Iron is really cool and Rust is awesome too!",
        "Metalman",
        chrono::offset::Utc::now(),
        Uuid::new_v4(),
    );
    db.add_post(p2);

    let handlers = Handlers::new(db);
    let json_content_middleware = JsonAfterMiddleware;

    let mut router = Router::new();
    router.get("/post_feed", handlers.post_feed, "post_feed");
    router.post("/post", handlers.post_post, "post_post");
    router.get("/post/:id", handlers.post, "post");

    let mut chain = Chain::new(router);
    chain.link_before(LoggerBefore::new());
    chain.link_after(json_content_middleware);
    chain.link_after(LoggerAfter::new());

    Iron::new(chain).http("localhost:8000").expect("Unable to start server");
}
