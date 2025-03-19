use crate::fileo::credentials::FileoCredentials;
use crate::member::config::MembershipsProviderConfig;
use crate::member::get_memberships_file_folder;
use crate::uda::credentials::UdaCredentials;
use crate::web::api::members_state::MembersState;
use crate::web::api::{fileo_controller, memberships_controller, uda_controller};
use crate::web::credentials_storage::CredentialsStorage;
use crate::web::server::Server;
use dto::uda::InstancesList;
use regex::Regex;
use rocket::{Build, Rocket};
use std::sync::Mutex;

pub struct ApiServer {}

impl ApiServer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Server for ApiServer {
    fn configure(&self, rocket_build: Rocket<Build>) -> Rocket<Build> {
        let members_provider_config = build_members_provider_config();
        let members_state = match MembersState::load_members(members_provider_config.folder()) {
            Ok(state) => state,
            Err(error) => {
                error!("{error:#?}");
                panic!("Initialization failed, aborting.");
            }
        };

        rocket_build
            .manage(members_provider_config)
            .manage(build_uda_configuration())
            .manage(Mutex::new(members_state))
            .manage(Mutex::new(CredentialsStorage::<FileoCredentials>::default()))
            .manage(Mutex::new(CredentialsStorage::<UdaCredentials>::default()))
            .manage(Mutex::new(InstancesList::default()))
            .mount(
                "/api/",
                routes![
                    memberships_controller::check_memberships,
                    memberships_controller::notify_members,
                    fileo_controller::login,
                    fileo_controller::download_memberships,
                    uda_controller::login,
                    uda_controller::retrieve_participants_to_check,
                    uda_controller::confirm_members,
                    uda_controller::list_instances,
                ],
            )
    }
}

fn build_members_provider_config() -> MembershipsProviderConfig {
    MembershipsProviderConfig::new(
        get_fileo_host(),
        get_download_link_regex(),
        get_memberships_file_folder().to_os_string(),
    )
}

#[cfg(not(feature = "demo"))]
fn get_fileo_host() -> String {
    "https://www.leolagrange-fileo.org".to_owned()
}

#[cfg(not(feature = "demo"))]
fn get_download_link_regex() -> Regex {
    Regex::new("https://www.leolagrange-fileo.org/clients/fll/telechargements/temp/.*?\\.csv")
        .unwrap()
}

#[cfg(not(feature = "demo"))]
fn build_uda_configuration() -> crate::uda::configuration::Configuration {
    crate::uda::configuration::Configuration::new(
        "https://reg.unicycling-software.com/tenants?locale=en".to_owned(),
    )
}

#[cfg(feature = "demo")]
fn get_fileo_host() -> String {
    crate::demo_mock_server::FILEO_MOCK_SERVER_URI
        .get()
        .unwrap()
        .clone()
}

#[cfg(feature = "demo")]
fn get_download_link_regex() -> Regex {
    Regex::new("http://.*?\\.csv").unwrap()
}

#[cfg(feature = "demo")]
fn build_uda_configuration() -> crate::uda::configuration::Configuration {
    let server_url = crate::demo_mock_server::UDA_MOCK_SERVER_URI
        .get()
        .unwrap()
        .clone();
    let url = format!("{server_url}/tenants?locale=en");

    crate::uda::configuration::Configuration::new(url)
}
