use actix_web::http::header::HeaderValue;
use sqlx::SqlitePool;

use crate::core::{
    constants::errors::{self, ErrorReason},
    db::{execute_query, fetch_all_as_json, AppliedQuery},
    util::Error,
};

use super::jwt::{generate_claims, Claims, RequestQuery, Sub};

pub async fn get_query_result_claims<'a>(
    query_claims: Claims,
    user_access: u8,
    db: &'a SqlitePool,
) -> Result<Claims, Error<'a>> {
    let dat: RequestQuery = serde_json::from_str(&query_claims.dat).unwrap();
    let response_claims_result = match query_claims.sub {
        Sub::MUTATE => {
            if user_access >= 2 {
                let mut transaction = db.begin().await.unwrap();

                match execute_query(
                    AppliedQuery::new(&dat.query).with_args(dat.parts),
                    &mut *transaction,
                )
                .await
                {
                    Ok(res) => {
                        let _ = &mut transaction.commit().await;

                        Ok(generate_claims(
                            serde_json::to_string(&res.rows_affected()).unwrap(),
                            Sub::DATA,
                        ))
                    }
                    Err(_) => {
                        let _ = &mut transaction.rollback().await;

                        Err(Error::new(errors::ERROR_UNSPECIFIED))
                    }
                }
            } else {
                Err(Error::new(errors::ERROR_FORBIDDEN).with_reason(ErrorReason::UserNotAllowed))
            }
        }
        Sub::FETCH => {
            if user_access >= 1 {
                let result =
                    fetch_all_as_json(AppliedQuery::new(&dat.query).with_args(dat.parts), &db)
                        .await
                        .unwrap();

                Ok(generate_claims(
                    serde_json::to_string(&result).unwrap(),
                    Sub::DATA,
                ))
            } else {
                Err(Error::new(errors::ERROR_FORBIDDEN).with_reason(ErrorReason::UserNotAllowed))
            }
        }
        _ => Err(Error::new(errors::ERROR_NOT_ACCEPTABLE).with_reason(ErrorReason::InvalidSubject)),
    };

    response_claims_result
}

pub fn get_header_value(header: Option<&HeaderValue>) -> Result<&str, &str> {
    match header {
        Some(hdr) => match hdr.to_str() {
            Ok(hdr_val) => Ok(hdr_val),
            Err(_) => Err(errors::ERROR_MALFORMED_HEADER),
        },
        None => Err(errors::ERROR_MISSING_HEADER),
    }
}
