use crate::fileo::credentials::FileoCredentials;
use crate::membership;
use crate::membership::check::check_members;
use crate::tools::email::send_email;
use crate::tools::log_message_and_return;
use crate::uda::credentials::UdaCredentials;
use crate::web::api::memberships_state::MembershipsState;
use dto::checked_member::CheckedMember;
use dto::csv_member::CsvMember;
use dto::email::Email;
use dto::member_to_check::MemberToCheck;
use dto::member_to_look_up::MemberToLookUp;
use dto::uda_member::UdaMember;
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::{Json, json};
use std::sync::Mutex;

/// Check members coming from a CSV file.
/// Return the result as JSON-encoded string,
/// within which each member having a valid membership has its last occurrence associated,
/// while each member having no valid membership has no element associated.
#[post(
    "/members/csv/check",
    format = "application/json",
    data = "<members_to_check>"
)]
pub async fn check_csv_members(
    memberships_state: &State<Mutex<MembershipsState>>,
    members_to_check: Json<Vec<CsvMember>>,
    _credentials: FileoCredentials,
) -> Result<String, String> {
    let result = check(memberships_state, members_to_check.into_inner())?;

    Ok(json!(result).to_string())
}

#[post(
    "/members/uda/check",
    format = "application/json",
    data = "<members_to_check>"
)]
pub async fn check_uda_members(
    memberships_state: &State<Mutex<MembershipsState>>,
    members_to_check: Json<Vec<UdaMember>>,
    _fileo_credentials: FileoCredentials,
    _uda_credentials: UdaCredentials,
) -> Result<String, String> {
    let result = check(memberships_state, members_to_check.into_inner())?;

    Ok(json!(result).to_string())
}

fn check<T: MemberToCheck>(
    memberships_state: &Mutex<MembershipsState>,
    members_to_check: Vec<T>,
) -> Result<Vec<CheckedMember<T>>, String> {
    let memberships_state = memberships_state.lock().map_err(log_message_and_return(
        "Couldn't acquire lock",
        "Error while checking members.",
    ))?;

    let memberships = memberships_state.memberships();
    let checked_members = check_members(memberships, members_to_check);

    Ok(checked_members)
}

/// Email all recipients specified as argument.
#[post("/members/notify", format = "application/json", data = "<email>")]
pub async fn notify_members(
    email: Json<Email>,
    _credentials: FileoCredentials,
) -> Result<(), String> {
    let recipients = email
        .recipients()
        .iter()
        .map(|email| email.as_ref())
        .collect::<Vec<&str>>();
    send_email(recipients.as_ref(), email.subject(), email.body())
        .await
        .map_err(log_message_and_return(
            "Couldn't send email",
            "Email has not been sent.",
        ))?;

    Ok(())
}

#[post(
    "/members/lookup",
    format = "application/json",
    data = "<member_to_look_up>"
)]
pub async fn look_member_up(
    memberships_state: &State<Mutex<MembershipsState>>,
    member_to_look_up: Json<MemberToLookUp>,
    _credentials: FileoCredentials,
) -> Result<String, Status> {
    let member_to_look_up = member_to_look_up.into_inner();

    if member_to_look_up.membership_num().is_none()
        && member_to_look_up.last_name().is_none()
        && member_to_look_up.first_name().is_none()
    {
        debug!("Can't look for empty member [member: {member_to_look_up:?}]");
        return Err(Status::BadRequest);
    }

    let memberships_state = memberships_state.lock().map_err(log_message_and_return(
        "Couldn't acquire lock",
        Status::InternalServerError,
    ))?;

    let memberships =
        membership::look_up::look_member_up(memberships_state.memberships(), &member_to_look_up);

    Ok(json!(memberships).to_string())
}

#[cfg(test)]
mod tests {
    use crate::fileo::credentials::FileoCredentials;
    use crate::uda::credentials::UdaCredentials;
    use crate::web::credentials_storage::CredentialsStorage;
    use std::sync::Mutex;

    fn initialize_fileo_login() -> (String, Mutex<CredentialsStorage<FileoCredentials>>) {
        let credentials =
            FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());

        let uuid = "e9af5e0f-c441-4bcd-bf22-31cc5b1f2f9e".to_owned();
        let mut storage = CredentialsStorage::<FileoCredentials>::default();
        storage.store(uuid.clone(), credentials);

        let storage_mutex = Mutex::new(storage);
        (uuid, storage_mutex)
    }

    fn initialize_uda_login() -> (String, Mutex<CredentialsStorage<UdaCredentials>>) {
        let credentials = UdaCredentials::new(
            "https://test.reg.unicycling-software.com".to_owned(),
            "login@test.com".to_owned(),
            "password".to_owned(),
        );

        let uuid = "e9af5e0f-c441-4bcd-bf22-31cc5b1f2f9e".to_owned();
        let mut storage = CredentialsStorage::<UdaCredentials>::default();
        storage.store(uuid.clone(), credentials);

        let storage_mutex = Mutex::new(storage);
        (uuid, storage_mutex)
    }

    mod check_members {
        use crate::membership::indexed_memberships::IndexedMemberships;
        use crate::web::api::memberships_controller::check_uda_members;
        use crate::web::api::memberships_controller::tests::{
            initialize_fileo_login, initialize_uda_login,
        };
        use crate::web::api::memberships_state::MembershipsState;
        use dto::checked_member::{CheckResult, CheckedMember};
        use dto::membership::tests::get_expected_membership;
        use dto::uda_member::UdaMember;
        use rocket::http::hyper::header::CONTENT_TYPE;
        use rocket::http::{ContentType, Header, Status};
        use rocket::local::asynchronous::Client;
        use rocket::serde::json::json;
        use std::sync::Mutex;

        #[async_test]
        async fn success() {
            let member_1 = UdaMember::new(
                1,
                Some("123456".to_owned()),
                "Jon".to_owned(),
                "Doe".to_owned(),
                "jon.doe@email.com".to_owned(),
                Some("Le club de test".to_owned()),
                true,
            );
            let member_2 = UdaMember::new(
                2,
                Some("654321".to_owned()),
                "Jonette".to_owned(),
                "Snow".to_owned(),
                "jonette.snow@email.com".to_owned(),
                None,
                false,
            );
            let members = vec![member_1.clone(), member_2.clone()];

            let (fileo_uuid, fileo_credentials_storage_mutex) = initialize_fileo_login();
            let (uda_uuid, uda_credentials_storage_mutex) = initialize_uda_login();

            let memberships_state = MembershipsState::new(
                None,
                IndexedMemberships::from(vec![get_expected_membership()]),
            );
            let memberships_state = Mutex::new(memberships_state);

            let rocket = rocket::build()
                .manage(fileo_credentials_storage_mutex)
                .manage(uda_credentials_storage_mutex)
                .manage(memberships_state)
                .mount("/", routes![check_uda_members]);

            let client = Client::tracked(rocket).await.unwrap();
            let request = client
                .post("/members/uda/check")
                .cookie((
                    crate::fileo::authentication::AUTHENTICATION_COOKIE,
                    fileo_uuid,
                ))
                .cookie((crate::uda::authentication::AUTHENTICATION_COOKIE, uda_uuid))
                .body(json!(members).to_string().as_bytes())
                .header(Header::new(
                    CONTENT_TYPE.to_string(),
                    ContentType::JSON.to_string(),
                ));

            let response = request.dispatch().await;
            assert_eq!(Status::Ok, response.status());

            let checked_members: Vec<CheckedMember<UdaMember>> =
                response.into_json().await.unwrap();
            assert_eq!(
                vec![
                    CheckedMember::new(member_1, CheckResult::Match(get_expected_membership())),
                    CheckedMember::new(member_2, CheckResult::NoMatch),
                ],
                checked_members
            )
        }
    }

    mod look_member_up {
        use crate::fileo::authentication::AUTHENTICATION_COOKIE;
        use crate::membership::indexed_memberships::IndexedMemberships;
        use crate::membership::indexed_memberships::tests::{
            jon_doe, jon_doe_previous_membership, jonette_snow, other_jon_doe,
        };
        use crate::web::api::memberships_controller::look_member_up;
        use crate::web::api::memberships_controller::tests::initialize_fileo_login;
        use crate::web::api::memberships_state::MembershipsState;
        use dto::member_to_look_up::MemberToLookUp;
        use dto::membership::Membership;
        use rocket::http::hyper::header::CONTENT_TYPE;
        use rocket::http::{ContentType, Header, Status};
        use rocket::local::asynchronous::Client;
        use rocket::serde::json::json;
        use std::sync::Mutex;

        #[async_test]
        async fn success() {
            let (fileo_uuid, fileo_credentials_storage_mutex) = initialize_fileo_login();

            let memberships_state = MembershipsState::new(
                None,
                IndexedMemberships::from(vec![
                    jon_doe(),
                    jon_doe_previous_membership(),
                    jonette_snow(),
                    other_jon_doe(),
                ]),
            );
            let memberships_state = Mutex::new(memberships_state);

            let rocket = rocket::build()
                .manage(fileo_credentials_storage_mutex)
                .manage(memberships_state)
                .mount("/", routes![look_member_up]);

            let client = Client::tracked(rocket).await.unwrap();

            let member_to_look_up =
                MemberToLookUp::new(Some(jon_doe().membership_number().to_owned()), None, None);
            let request = client
                .post("/members/lookup")
                .cookie((AUTHENTICATION_COOKIE, fileo_uuid))
                .body(json!(member_to_look_up).to_string().as_bytes())
                .header(Header::new(
                    CONTENT_TYPE.to_string(),
                    ContentType::JSON.to_string(),
                ));

            let response = request.dispatch().await;
            assert_eq!(Status::Ok, response.status());

            let matching_memberships: Vec<Membership> = response.into_json().await.unwrap();
            assert_eq!(
                vec![jon_doe_previous_membership(), jon_doe()],
                matching_memberships
            )
        }

        #[async_test]
        async fn bad_request() {
            let (fileo_uuid, fileo_credentials_storage_mutex) = initialize_fileo_login();

            let memberships_state = MembershipsState::new(
                None,
                IndexedMemberships::from(vec![
                    jon_doe(),
                    jon_doe_previous_membership(),
                    jonette_snow(),
                    other_jon_doe(),
                ]),
            );
            let memberships_state = Mutex::new(memberships_state);

            let rocket = rocket::build()
                .manage(fileo_credentials_storage_mutex)
                .manage(memberships_state)
                .mount("/", routes![look_member_up]);

            let client = Client::tracked(rocket).await.unwrap();

            let member_to_look_up = MemberToLookUp::new(None, None, None);
            let request = client
                .post("/members/lookup")
                .cookie((AUTHENTICATION_COOKIE, fileo_uuid))
                .body(json!(member_to_look_up).to_string().as_bytes())
                .header(Header::new(
                    CONTENT_TYPE.to_string(),
                    ContentType::JSON.to_string(),
                ));

            let response = request.dispatch().await;
            assert_eq!(Status::BadRequest, response.status());
        }
    }
}
