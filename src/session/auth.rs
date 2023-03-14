use super::{Session, extract_db, Error as GeneralError};
use serde::{Serialize, Deserialize};
use jsonwebtoken::{Algorithm, EncodingKey, DecodingKey, Validation};
use rand::Rng;
use pwhash::bcrypt;

const NAME_CHARS: &str = "QAZWSXEDCRFVTGBYHNUJMIKOLPqazwsxedcrfvtgbyhnujmikolp1234567890_";

pub enum Auth {
    Valid {
        info: AuthInfo
    },
    Invalid(String)
}

impl Auth {
    pub fn as_result<E, Mapper>(&self, map_error: Mapper) -> Result<&AuthInfo, E>
    where Mapper: FnOnce(&String) -> E
    {
        match self {
            Auth::Valid { info } => Ok(info),
            Auth::Invalid(reason) => Err(map_error(reason))
        }
    }
}

pub struct AuthInfo {
    pub name: String
}

pub struct Tokens {
    pub access: String,
    pub key: String,
    keys: Keys
}

#[derive(Clone)]
pub struct Keys {
    pub access: String,
    pub key: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub name: String,
    pub key: String,
    pub exp: usize,
}

#[derive(Serialize, Deserialize)]
pub struct KeyClaims {
    pub key: String,
    pub exp: usize,
}

impl Tokens {
    pub fn new(access: String, key: String, keys: Keys) -> Self {
        Self {
            access,
            key,
            keys
        }
    }

    pub fn from_auth(info: AuthInfo, keys: Keys) -> Result<Self, String> {
        let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
        .expect("valid timestamp")
        .timestamp() as usize;

        let mut rng = rand::thread_rng();
        let key: String = (0..20).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect();

        let access_claims = AccessClaims {
            exp,
            key: key.clone(),
            name: info.name
        };

        let key_claims = KeyClaims {
            exp,
            key
        };

        let header = jsonwebtoken::Header::new(Algorithm::HS512);

        let access_token = jsonwebtoken::encode(&header, &access_claims, &EncodingKey::from_secret(&keys.access.as_bytes()))
        .map_err(|error| error.to_string())?;

        let key_token = jsonwebtoken::encode(&header, &key_claims, &EncodingKey::from_secret(&keys.key.as_bytes()))
        .map_err(|error| error.to_string())?;

        Ok(Self {
            keys,
            access: access_token,
            key: key_token
        })
    }

    pub fn into_auth(self) -> Auth {
        let keys = self.keys;

        let access_claims = match jsonwebtoken::decode::<AccessClaims>(
            &self.access,
            &DecodingKey::from_secret(keys.access.as_bytes()),
            &Validation::new(Algorithm::HS512)
        ) {
            Ok(claims) => claims,
            Err(error) => return Auth::Invalid(format!("access token - {error}"))
        }.claims;

        let key_claims = match jsonwebtoken::decode::<KeyClaims>(
            &self.key,
            &DecodingKey::from_secret(keys.key.as_bytes()),
            &Validation::new(Algorithm::HS512)
        ) {
            Ok(claims) => claims,
            Err(error) => return Auth::Invalid(format!("key token - {error}"))
        }.claims;

        if key_claims.key != access_claims.key {
            return Auth::Invalid("Keys don't match".to_string());
        }

        Auth::Valid {
            info: AuthInfo {
                name: access_claims.name
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RegisterError {
    #[error("invalid username characters")]
    InvaildNameChars,
    #[error("username is too long or too short")]
    BadNameLength,
    #[error("password is too short")]
    TooShortPassword,
    #[error("too long password")]
    TooLongPassword,
    #[error("general error: {0}")]
    General(GeneralError),
    #[error("username already taken")]
    NameTaken,
    #[error("email already taken")]
    EmailTaken,
    #[error("failed to generate tokens: {0}")]
    TokenGenerationFailed(String),
    #[error("failed to hash password: {0}")]
    Hashing(String)
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("failed to generate tokens: {0}")]
    TokenGenerationFailed(String),
    #[error("general error: {0}")]
    General(GeneralError),
    #[error("invalid credentials")]
    InvalidCredentials
}

fn hash_password(password: &str) -> Result<String, String> {
    bcrypt::hash(password)
    .map_err(|error| error.to_string())
}

pub fn compare_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash)
}

impl<LC> Session<LC> {
    pub async fn login(&self, name: &str, password: &str) -> Result<Tokens, LoginError> {
        let tokens = Tokens::from_auth(AuthInfo {
            name: name.to_string()
        }, self.auth_keys.clone()).map_err(LoginError::TokenGenerationFailed)?;

        extract_db!(self, db_pool, db_pool_cloned);

        let user = db_pool.get_user(name).await.map_err(|error| LoginError::General(GeneralError::Db(error)))?;
        if !compare_password(password, user.password_hash.as_str()) {
            return Err(LoginError::InvalidCredentials);
        }

        Ok(tokens)
    }

    pub async fn register(&self, name: &str, email: &str, password: &str) -> Result<Tokens, RegisterError> {
        for char in name.chars(){
            if !NAME_CHARS.contains(char) {
                return Err(RegisterError::InvaildNameChars);
            }
        }
        if name.len() < 3 || name.len() > 20 {
            return Err(RegisterError::BadNameLength);
        }
        if password.len() < 7 {
            return Err(RegisterError::TooShortPassword);
        }
        if password.len() > 50 {
            return Err(RegisterError::TooLongPassword);
        }

        extract_db!(self, db_pool, db_pool_cloned);
        let uniqueness = db_pool.check_if_unique_credentials(name, email).await.map_err(|error| RegisterError::General(GeneralError::Db(error)))?;
        if !uniqueness.email {
            return Err(RegisterError::EmailTaken);
        }
        if !uniqueness.name {
            return Err(RegisterError::NameTaken);
        }

        let tokens = Tokens::from_auth(AuthInfo {
            name: name.to_string()
        }, self.auth_keys.clone()).map_err(RegisterError::TokenGenerationFailed)?;

        let password_hash = hash_password(password).map_err(RegisterError::Hashing)?;
        db_pool.create_user(name, email, password_hash.as_str()).await.map_err(|error| RegisterError::General(GeneralError::Db(error)))?;

        Ok(tokens)
    }
}