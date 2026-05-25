use axum::{body::Body, extract::State, http::{Request, header}, middleware::Next, response::Response};

use crate::{errors::AppError, state::AppState, utils::paseto::{create_access_token, verify_access_token}};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub roles: Vec<String>
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next
) -> Result<Response, AppError> {
    let token = extract_bearer_token(&req)?;
    let claims = verify_access_token(token, &state.config.auth.paseto_key)?;

    let user_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::InvalidToken)?;

    req.extensions_mut().insert(AuthUser{
        id: user_id,
        roles: claims.roles
    });

    Ok(next.run(req).await)
}

fn extract_bearer_token(req: &Request<Body>) -> Result<&str, AppError>{
    let header_value = req
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(AppError::Unauthorized)?
        .to_str()
        .map_err(|_| AppError::Unauthorized)?;

    header_value
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)
}

// pub fn require_roles(
//     allowed: &'static [&'static str],
// ) -> impl Fn(AuthUser) -> Result<AuthUser, AppError> + Clone {
//     move |user: AuthUser| {
//         if allowed.contains(&user.role.as_str()) {
//             Ok(user)
//         } else {
//             Err(AppError::Forbidden)
//         }
//     }
// }

pub fn require_roles(
    allowed: &'static [&'static str]
) -> impl Fn(AuthUser) -> Result<AuthUser, AppError> + Clone {
    move |user: AuthUser| {
        // Manual
        // let mut has_role = false;
        // for r in &user.roles {
        //     if allowed.contains(&r.as_str()) {
        //         has_role = true;
        //         break;
        //     }
        // }

        let has_role = user.roles.iter().any(|r| {
            allowed.contains(&r.as_str())
        });
        
        if has_role{
            Ok(user)
        } else {
            Err(AppError::Forbidden)
        }
    }
}
/*
let numbers = vec![1, 2, 3, 4, 5];

// any() — apakah ada satu yang memenuhi syarat?
numbers.iter().any(|n| *n > 3)     // true (ada 4 dan 5)

// all() — apakah SEMUA memenuhi syarat?
numbers.iter().all(|n| *n > 0)     // true (semua positif)
numbers.iter().all(|n| *n > 3)     // false (1,2,3 tidak > 3)

// find() — ambil element pertama yang memenuhi syarat
numbers.iter().find(|n| **n > 3)   // Some(4)

// filter() — ambil semua yang memenuhi syarat
numbers.iter().filter(|n| **n > 3) // [4, 5]

// map() — ubah setiap element
numbers.iter().map(|n| n * 2)      // [2, 4, 6, 8, 10]

// count() — hitung berapa yang memenuhi syarat
numbers.iter().filter(|n| **n > 3).count() // 2
*/

//  iter() menghasilkan referensi ke setiap element
//  tipe setiap element: &i32  ← satu lapis referensi
// numbers.iter().any(|n| *n > 3)
//                  ^  ^^
//                  |   dereference — ambil i32 dari &i32
//                  n bertipe &i32
// numbers.iter().filter(|n| **n > 3)
//                        ^^^
//                        n bertipe &&i32  ← dua lapis referensi! Karena filter() secara internal memberikan referensi lagi ke element yang sudah merupakan referensi:

// contoh:
/*
let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// Dengan loop manual — butuh variabel sementara
let mut result = vec![];
for n in &numbers {
    if n % 2 == 0 {
        result.push(n * 10);
    }
}

// Dengan iterator — satu baris, tidak perlu variabel sementara
let result: Vec<i32> = numbers.iter()
    .filter(|n| *n % 2 == 0)  // ambil yang genap
    .map(|n| n * 10)           // kalikan 10
    .collect();                // kumpulkan hasilnya

// hasil: [20, 40, 60, 80, 100]
*/

// if uses jwt:
// pub async fn auth_middleware(
//     State(state): State<AppState>,
//     mut req: Request<Body>,
//     next: Next
// ) -> Result<Response, AppError> {
//     let token = extract_bearer_token(&req)?;
//     let claims = verify_token(token, &state.config.jwt_secret)?;

//     let user_id: i64 = claims
//         .sub
//         .parse()
//         .map_err(|_| AppError::InvalidToken)?;
    
//     req.extensions_mut().insert(AuthUser {
//         id: user_id,
//         roles: claims.roles
//     });
    
//     Ok(next.run(req).await)
// }