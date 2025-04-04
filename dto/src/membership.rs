use chrono::NaiveDate;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Serialize, Deserialize, Getters, PartialEq, Eq, Hash, Clone)]
pub struct Membership {
    name: String,
    first_name: String,
    gender: String,
    birthdate: Option<NaiveDate>,
    age: Option<u8>,
    membership_number: String,
    email_address: String,
    payed: bool,
    end_date: NaiveDate,
    expired: bool,
    club: String,
    structure_code: String,
}

impl Membership {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        first_name: String,
        gender: String,
        birthdate: Option<NaiveDate>,
        age: Option<u8>,
        membership_number: String,
        email_address: String,
        payed: bool,
        end_date: NaiveDate,
        expired: bool,
        club: String,
        structure_code: String,
    ) -> Self {
        Self {
            name,
            first_name,
            gender,
            birthdate,
            age,
            membership_number,
            email_address,
            payed,
            end_date,
            expired,
            club,
            structure_code,
        }
    }
}

impl PartialOrd for Membership {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Membership {
    fn cmp(&self, other: &Self) -> Ordering {
        self.end_date
            .cmp(&other.end_date)
            .then(self.membership_number.cmp(&other.membership_number))
            .then(self.name.cmp(&other.name))
            .then(self.first_name.cmp(&other.first_name))
    }
}

#[cfg(any(test, feature = "test"))]
pub mod tests {
    use super::*;
    use parameterized::{ide, parameterized};

    ide!();

    impl Membership {
        pub fn new_test(end_date: NaiveDate) -> Self {
            Membership {
                name: "".to_string(),
                first_name: "".to_string(),
                gender: "".to_string(),
                birthdate: None,
                age: None,
                membership_number: "".to_string(),
                email_address: "".to_string(),
                payed: false,
                end_date,
                expired: false,
                club: "".to_string(),
                structure_code: "".to_string(),
            }
        }
    }

    const HEADER: &str = "Nom d'usage;Prénom;Sexe;Date de Naissance;Age;Numéro d'adhérent;Email;Réglé;Date Fin d'adhésion;Adherent expiré;Nom de structure;Code de structure";
    const MEMBERSHIP_AS_CSV: &str =
        "Doe;Jon;H;01-02-1980;45;123456;email@address.com;Oui;30-09-2025;Non;My club;Z01234";
    pub const MEMBER_NAME: &str = "Doe";
    pub const MEMBER_FIRST_NAME: &str = "Jon";
    pub const MEMBERSHIP_NUMBER: &str = "123456";
    const MALFORMED_MEMBERSHIP_AS_CSV: &str =
        "Doe;Jon;H;01-02-1980;45;123456;email@address.com;Oops;30-09-2025;Non;My club;Z01234";

    pub fn get_expected_membership() -> Membership {
        Membership {
            name: "Doe".to_string(),
            first_name: "Jon".to_string(),
            gender: "H".to_string(),
            birthdate: NaiveDate::from_ymd_opt(1980, 2, 1),
            age: Some(45),
            membership_number: MEMBERSHIP_NUMBER.to_string(),
            email_address: "email@address.com".to_string(),
            payed: true,
            end_date: NaiveDate::from_ymd_opt(2025, 9, 30).unwrap(),
            expired: false,
            club: "My club".to_string(),
            structure_code: "Z01234".to_string(),
        }
    }

    pub fn get_membership_as_csv() -> String {
        format!("{HEADER}\n{MEMBERSHIP_AS_CSV}")
    }

    pub fn get_malformed_membership_as_csv() -> String {
        format!("{HEADER}\n{MALFORMED_MEMBERSHIP_AS_CSV}")
    }

    #[parameterized(
        end_dates = {
        ((2020, 10, 12), (2020, 11, 12)),
        ((2020, 11, 12), (2020, 10, 12)),
        ((2020, 11, 12), (2020, 11, 12)),
        },
        expected_result = {
        Ordering::Less,
        Ordering::Greater,
        Ordering::Equal,
        }
    )]
    fn should_sort_memberships(
        end_dates: ((i32, u32, u32), (i32, u32, u32)),
        expected_result: Ordering,
    ) {
        let ((y1, m1, d1), (y2, m2, d2)) = end_dates;
        let member1 = Membership::new_test(NaiveDate::from_ymd_opt(y1, m1, d1).unwrap());
        let member2 = Membership::new_test(NaiveDate::from_ymd_opt(y2, m2, d2).unwrap());
        assert_eq!(Some(expected_result), member1.partial_cmp(&member2));
    }
}
