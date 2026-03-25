use crate::{infrastructure::db::DBPool, handlers::auth::RegisterRequest, models::{user_roles::DEFAULT_ROLE, users::User}};

pub async fn find_user_with_role_by_email(pool: &DBPool, email: &str) -> sqlx::Result<Option<(User, Option<String>)>>{
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, name, email, password, photo, phone, created_at, updated_at
        FROM public.users
        WHERE email = $1
        "#,
        email
    ).fetch_optional(pool).await?;

    if let Some(user) = user {
        let role: Option<(String, )> = sqlx::query_as(
            r#"
            SELECT r.name
            FROM public.user_roles ur
            JOIN public.roles r ON ur.role_id = r.id
            WHERE ur.user_id = $1
            "#
        ).bind(user.id).fetch_optional(pool).await?;
        Ok(Some((user, role.map(|t| t.0))))
    } else {
        Ok(None)
    }
    // equivalent
    // match user {
    //     Some(user) => { ... }
    //     None => { ... }
    // }
}

    // pub name: String,
    // pub email: String,
    // pub password: String,
    // pub photo: Option<String>,
    // pub phone: Option<String>,

pub async fn create_user(pool: &DBPool, payload: &RegisterRequest) -> sqlx::Result<User, sqlx::Error>{
    let mut tx = pool.begin().await?;
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO public.users (name, email, password, photo, phone)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, email, password, photo, phone, created_at, updated_at
        "#,
        payload.name,
        payload.email,
        payload.password,
        payload.photo,
        payload.phone
    ).fetch_one(&mut *tx).await?;

    sqlx::query!(
        r#"
        INSERT INTO public.user_roles (user_id, role_id)
        VALUES ($1, (SELECT id FROM public.roles WHERE name = $2))
        "#,
        user.id,
        DEFAULT_ROLE
    ).execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(user)
}