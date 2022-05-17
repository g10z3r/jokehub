use argon2::Config;
use chrono::{NaiveDateTime, Utc};
use lazy_static::lazy_static;
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::validate_query;
use crate::errors::HubError;

lazy_static! {
    static ref ROLE_USER: &'static str = "user";
    static ref ROLE_ADMIN: &'static str = "admin";
}

#[derive(Clone, Validate, Deserialize)]
pub struct NewUser {
    #[validate(
        length(min = 4, max = 10, message = "Lenght is invalid"),
        custom(function = "validate_query", message = "Invalid format")
    )]
    pub username: String,

    #[validate(
        length(min = 8, max = 20, message = "Lenght is invalid"),
        custom(function = "validate_query", message = "Invalid format")
    )]
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,

    pub username: String,
    pub role: String,

    pub hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<NewUser> for User {
    fn from(nu: NewUser) -> Self {
        User {
            id: Uuid::new_v4().to_string(),
            username: nu.username,
            role: String::from(ROLE_USER.clone()),
            hash: nu.password,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl<'a> User {
    // Верификация пароля
    pub fn password_verify(&self, password: &[u8]) -> Result<bool, HubError> {
        argon2::verify_encoded(&self.hash, password).map_err(|err| {
            HubError::new_internal("Failed verify password", Some(Vec::new()))
                .add(format!("{}", err))
        })
    }

    // Создание хеша пароля
    pub fn password_hashing(&mut self) -> Result<User, HubError> {
        let salt: [u8; 32] = rand::thread_rng().gen();
        let config = Config::default();

        self.hash = argon2::hash_encoded(self.hash.as_bytes(), &salt, &config).map_err(|err| {
            HubError::new_internal("Failed create password hash", Some(Vec::new()))
                .add(format!("{}", err))
        })?;

        Ok(self.clone())
    }
}

pub mod security {
    use chrono::prelude::*;
    use jsonwebtoken::TokenData;
    use jsonwebtoken::{errors::ErrorKind as JwtErrorKind, DecodingKey, EncodingKey, Validation};
    use rocket::{
        request, request::FromRequest, request::Outcome, serde::DeserializeOwned, Request,
    };
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use crate::errors::{ErrorKind, HubError, UnauthorizedErrorKind};

    const SECRET: &str = "secret297152aebda7";

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct AccessClaims {
        access_uuid: Uuid,
        username: String,
        role: String,

        #[serde(with = "jwt_numeric_date")]
        exp: DateTime<Utc>,
    }

    impl AccessClaims {
        fn new(username: String, role: String) -> Self {
            // Задаю срок жизни access токена
            let exp = Utc::now() + chrono::Duration::minutes(15);

            // Нормализация к временным меткам UNIX
            let exp = exp
                .date()
                .and_hms_milli(exp.hour(), exp.minute(), exp.second(), 0);

            AccessClaims {
                access_uuid: Uuid::new_v4(),
                username,
                role,
                exp,
            }
        }

        pub fn get_username(&self) -> String {
            return self.username.clone();
        }

        pub fn get_role(&self) -> String {
            return self.role.clone();
        }
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct RefreshClaims {
        refresh_uuid: Uuid,
        access_uuid: Uuid,

        #[serde(with = "jwt_numeric_date")]
        refresh_exp: DateTime<Utc>,

        #[serde(with = "jwt_numeric_date")]
        access_exp: DateTime<Utc>,
    }

    impl RefreshClaims {
        fn new(ac: &AccessClaims) -> Self {
            // Задаю срок жизни refresh токена
            let exp = Utc::now() + chrono::Duration::days(7);

            // Нормализация к временным меткам UNIX
            let exp = exp
                .date()
                .and_hms_milli(exp.hour(), exp.minute(), exp.second(), 0);

            RefreshClaims {
                refresh_uuid: Uuid::new_v4(),
                access_uuid: ac.access_uuid,
                refresh_exp: exp,
                access_exp: ac.exp,
            }
        }
    }

    mod jwt_numeric_date {
        //! Сериализация DateTime<Utc> для соответствия спецификации JWT (RFC 7519 раздел 2, "Numeric Date")
        use chrono::{DateTime, TimeZone, Utc};
        use serde::{self, Deserialize, Deserializer, Serializer};

        /// Сериализирует DateTime<Utc> в отметку времени Unix (миллисекунды с 1970/1/1T00:00:00T)
        pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let timestamp = date.timestamp();
            serializer.serialize_i64(timestamp)
        }

        /// Попытки десериализовать i64 и использовать в качестве временной метки Unix
        pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
        where
            D: Deserializer<'de>,
        {
            Utc.timestamp_opt(i64::deserialize(deserializer)?, 0)
                .single() // Если есть несколько или нет действительных значений DateTimes из метки времени, возвращаю None
                .ok_or_else(|| serde::de::Error::custom("invalid Unix timestamp value"))
        }
    }

    pub struct AuthGuard(pub AccessClaims);

    #[rocket::async_trait]
    impl<'r> FromRequest<'r> for AuthGuard {
        type Error = HubError;

        async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
            match request.headers().get_one("Authorization") {
                Some(at) => {
                    let split = at.split(" ");
                    let vec = split.collect::<Vec<&str>>();

                    if vec.len() != 2 {
                        let kind = ErrorKind::Unauthorized(UnauthorizedErrorKind::Generic(
                            "Token is in invalid format",
                        ));
                        let error = HubError::new(kind);

                        Outcome::Failure((error.get_status(), error))
                    } else {
                        let token = Tokens::decode_token::<AccessClaims>(vec[1]);

                        match token {
                            Ok(t) => Outcome::Success(AuthGuard(t.claims)),
                            Err(err) => Outcome::Failure((err.get_status(), err)),
                        }
                    }
                }

                None => {
                    let kind = ErrorKind::Unauthorized(UnauthorizedErrorKind::TokenMissing);
                    let err = HubError::new(kind);

                    Outcome::Failure((err.get_status(), err))
                }
            }
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct Tokens {
        pub access_token: String,
        pub refresh_token: String,
    }

    impl<'a> Tokens {
        pub fn new(username: String, role: String) -> Result<Tokens, HubError> {
            let access_claims = AccessClaims::new(username, role);
            let refresh_claims = RefreshClaims::new(&access_claims);

            let tokens = Tokens {
                access_token: Tokens::encode_access_token(&access_claims)?,
                refresh_token: Tokens::encode_refresh_token(&refresh_claims)?,
            };

            Ok(tokens)
        }

        fn encode_access_token(ac: &AccessClaims) -> Result<String, HubError> {
            jsonwebtoken::encode(
                &jsonwebtoken::Header::default(),
                ac,
                &EncodingKey::from_secret(SECRET.as_ref()),
            )
            .map_err(|err| {
                HubError::new_internal("Failed to create access token", Some(Vec::new()))
                    .add(format!("{}", err))
            })
        }

        fn encode_refresh_token(rc: &RefreshClaims) -> Result<String, HubError> {
            jsonwebtoken::encode(
                &jsonwebtoken::Header::default(),
                rc,
                &EncodingKey::from_secret(SECRET.as_ref()),
            )
            .map_err(|err| {
                HubError::new_internal("Failed to create refresh token", Some(Vec::new()))
                    .add(format!("{}", err))
            })
        }

        pub fn decode_token<T>(token: &'a str) -> Result<TokenData<T>, HubError>
        where
            T: DeserializeOwned,
        {
            match jsonwebtoken::decode::<T>(
                &token,
                &DecodingKey::from_secret(SECRET.as_ref()),
                &Validation::default(),
            ) {
                Ok(token_data) => Ok(token_data),
                Err(err) => match *err.kind() {
                    JwtErrorKind::ExpiredSignature => {
                        let kind = ErrorKind::Unauthorized(UnauthorizedErrorKind::TokenExpired);

                        Err(HubError::new(kind))
                    }
                    _ => {
                        let kind = ErrorKind::Unauthorized(UnauthorizedErrorKind::Generic(
                            "Faild to decode token",
                        ));
                        let error = HubError::new(kind).add(format!("{}", err));

                        Err(error)
                    }
                },
            }
        }
    }
}