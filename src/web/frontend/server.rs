use rocket::{Build, Rocket};
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;
use crate::web::server::Server;
use crate::web::frontend::frontend_controller;

pub struct FrontendServer {

}

impl FrontendServer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Server for FrontendServer {
    fn configure(&self, rocket_build: Rocket<Build>) -> Rocket<Build> {
        rocket_build.mount("/", routes![
            frontend_controller::index,
            frontend_controller::hello
        ])
            .mount("/", FileServer::from("./public/static"))
            .register("/", catchers![frontend_controller::not_found])
            .attach(Template::fairing())
    }
}
