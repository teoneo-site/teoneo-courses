
/*
JSON:
 data: { 
    username
    email
    courses: {
        id,
        title, 
        brief_description
    }
}
*/

use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}};

use crate::{common::token::Claims, controllers, handlers::{self, ErrorTypes}, AppState};

pub async fn get_user_info_and_courses(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Response, Response> {
   let user_id = claims.id;

   match controllers::user::get_user_info(&state, user_id).await {
        Ok(info) => {
            let mut json_obj = serde_json::Value::Object(serde_json::Map::new());
            json_obj["data"] = serde_json::to_value(&info).unwrap(); // Why would it panic?
            return Ok((StatusCode::OK, axum::Json(json_obj)).into_response())
        },
        Err(why) => {
            eprintln!("Why failed: {}", why);
            eprintln!("AAAA: {}", why.to_string());
            if why.to_string() == "User does not exist" {
                return Err((
                StatusCode::BAD_REQUEST,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "User does not exist",
                )), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
            }
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Something happened",
                )), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
   }
}