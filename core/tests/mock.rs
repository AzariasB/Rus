mod prepare;

use rus_core::{Mutation, Query};
use prepare::prepare_mock_db;

#[tokio::test]
async fn main() {
    let db = &prepare_mock_db();

    {
        let redirection = Query::find_redirection_by_id(db, 1).await.unwrap().unwrap();

        assert_eq!(redirection.id, 1);
    }

    {
        let redirection = Query::find_redirection_by_id(db, 5).await.unwrap().unwrap();

        assert_eq!(redirection.id, 5);
    }

    {
        let redirection = Mutation::create_redirection(
            db,
            "https://example.com/created".to_string(),
        )
            .await
            .unwrap();

        assert_eq!(
            redirection.long_url,
            sea_orm::ActiveValue::Unchanged("https://example.com/created".to_string())
        );
    }

    {
        let redirection = Mutation::update_redirection_by_id(
            db,
            1,
            "https://example.com/updated".to_string(),
        )
            .await
            .unwrap();

        assert_eq!(
            redirection.long_url,
            "https://example.com/updated".to_string()
        );
    }

    {
        let result = Mutation::delete_redirection(db, 5).await.unwrap();

        assert_eq!(result.rows_affected, 1);
    }

    {
        let result = Mutation::delete_all_redirections(db).await.unwrap();

        assert_eq!(result.rows_affected, 5);
    }
}
