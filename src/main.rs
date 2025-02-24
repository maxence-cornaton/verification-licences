mod member;
mod server;

#[macro_use]
extern crate rocket;

use std::collections::{BTreeSet, HashMap};
use std::ffi::OsString;
use std::sync::Mutex;
use chrono::Local;
use rocket::State;
use rocket::time::macros::datetime;
use crate::member::download::download_members_list;
use crate::member::import_from_file::{find_file, import_from_file};
use crate::member::member::Member;
use crate::server::members_state::MembersState;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/members")]
async fn members(members_state: &State<Mutex<MembersState>>) -> Result<String, String> {
    let members_state = members_state.lock().unwrap();
    let filename: &Option<OsString> = members_state.filename();
    if filename.is_none() {
        Err("Can't find file.".to_string())
    } else {
        let members_by_membership = import_from_file(filename.as_ref().unwrap());
        Ok(format!("{:#?}", members_by_membership))
    }
}

#[post("/members")]
async fn update_members(members_state: &State<Mutex<MembersState>>) {
    let (datetime, filename) = match download_members_list().await {
        Ok((datetime, filename)) => { (datetime, filename) }
        Err(_) => { panic!("Oops") }
    };

    let mut members_state = members_state.lock().unwrap();
    members_state.set_filename(filename);
    members_state.set_last_update(datetime.clone());
}

#[launch]
fn rocket() -> _ {
    let mut members_state = MembersState::default();
    let file_details = find_file();
    match file_details {
        Some((date, filename)) => {
            members_state.set_last_update(date);
            members_state.set_filename(filename);
        }
        None => {}
    }

    rocket::build()
        .manage(Mutex::new(members_state))
        .mount("/", routes![index, members, update_members])
}
