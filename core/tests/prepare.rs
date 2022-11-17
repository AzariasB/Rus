use ::entity::redirection;
use sea_orm::*;

#[cfg(feature = "mock")]
pub fn prepare_mock_db() -> DatabaseConnection {
    MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results(vec![
            vec![redirection::Model {
                id: 1,
                long_url: "https://example.com/".to_owned(),
                short_url: "abcde".to_owned(),
                creation_date: Default::default(),
                expiration_date: None,
                last_access_date: Default::default(),
                ip_address: "".to_string(),
            }],
            vec![redirection::Model {
                id: 5,
                long_url: "https://example.com/".to_owned(),
                short_url: "eeeee".to_owned(),
                creation_date: Default::default(),
                expiration_date: None,
                last_access_date: Default::default(),
                ip_address: "".to_string(),
            }],
            vec![redirection::Model {
                id: 6,
                long_url: "https://example.com/".to_owned(),
                short_url: "fffff".to_owned(),
                creation_date: Default::default(),
                expiration_date: None,
                last_access_date: Default::default(),
                ip_address: "".to_string(),
            }],
            vec![redirection::Model {
                id: 1,
                long_url: "https://example.com/".to_owned(),
                short_url: "ggggg".to_owned(),
                creation_date: Default::default(),
                expiration_date: None,
                last_access_date: Default::default(),
                ip_address: "".to_string(),
            }],
            vec![redirection::Model {
                id: 1,
                long_url: "https://example.com/".to_owned(),
                short_url: "hhhhh".to_owned(),
                creation_date: Default::default(),
                expiration_date: None,
                last_access_date: Default::default(),
                ip_address: "".to_string(),
            }],
            vec![redirection::Model {
                id: 5,
                long_url: "https://example.com/created".to_owned(),
                short_url: "iiiiii".to_owned(),
                creation_date: Default::default(),
                expiration_date: None,
                last_access_date: Default::default(),
                ip_address: "".to_string(),
            }],
        ])
        .append_exec_results(vec![
            MockExecResult {
                last_insert_id: 6,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 6,
                rows_affected: 5,
            },
        ])
        .into_connection()
}
