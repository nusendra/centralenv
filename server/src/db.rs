use crate::Db;

pub async fn seed_admin(db: &Db) -> anyhow::Result<()> {
    let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) as count FROM admin_user")
        .fetch_one(db)
        .await?;

    if count == 0 {
        let username = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());
        let password = std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD env var required on first run");
        let hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST)?;
        sqlx::query!(
            "INSERT INTO admin_user (username, password_hash) VALUES (?, ?)",
            username,
            hash
        )
        .execute(db)
        .await?;
        tracing::info!("Created admin user: {}", username);
    }

    Ok(())
}
