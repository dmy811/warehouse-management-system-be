// pub async fn create_user(pool: &DBPool, payload: &RegisterRequest) -> sqlx::Result<User, sqlx::Error>{
//     let mut tx = pool.begin().await?;
//     let user = sqlx::query_as!(
//         User,
//         r#"
//         INSERT INTO public.users (name, email, password, photo, phone)
//         VALUES ($1, $2, $3, $4, $5)
//         RETURNING id, name, email, password, photo, phone, created_at, updated_at
//         "#,
//         payload.name,
//         payload.email,
//         payload.password,
//         payload.photo,
//         payload.phone
//     ).fetch_one(&mut *tx).await?;

//     sqlx::query!(
//         r#"
//         INSERT INTO public.user_roles (user_id, role_id)
//         VALUES ($1, (SELECT id FROM public.roles WHERE name = $2))
//         "#,
//         user.id,
//         DEFAULT_ROLE
//     ).execute(&mut *tx).await?;

//     tx.commit().await?;
//     Ok(user)
// }