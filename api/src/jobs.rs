use crate::AppState;
use log::{info, warn};
use rus_core::chrono::Utc;
use rus_core::Query;
use tokio_schedule::{every, Job};

pub async fn remove_expired_redirections(app_state: AppState) {
    let conn = &app_state.conn;
    let every_day = every(1).minutes().in_timezone(&Utc).perform(|| async {
        let res = Query::delete_outdated_redirections(conn).await;
        match res {
            Err(err) => warn!(
                "Failed to remove outdated redirection from database : {}",
                err
            ),
            Ok(res) => {
                if !res.is_empty() {
                    info!("Removed {} redirections from database", res.len())
                }
            }
        }
    });
    every_day.await;
}
