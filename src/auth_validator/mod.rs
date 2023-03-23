use jsonwebtoken::{Algorithm, EncodingKey, DecodingKey, Validation};
use serde::{Serialize, Deserialize};

pub enum Auth {
    Valid {
        info: AuthInfo
    },
    Invalid(String),
}

impl Auth {
    pub fn as_result(&self) -> Result<&AuthInfo, String>
    {
        match self {
            Auth::Valid { info } => Ok(info),
            Auth::Invalid(reason) => Err(reason.clone())
        }
    }
}

pub struct AuthInfo {
    pub name: String,
    pub activity_table_id: String
}

#[derive(Clone, Debug)]
pub struct Keys {
    pub access: String,
    pub key: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub name: String,
    pub activity_table_id: String,
    pub key: String,
    pub exp: usize,
}

#[derive(Serialize, Deserialize)]
pub struct KeyClaims {
    pub key: String,
    pub exp: usize,
}

pub struct Tokens {
    pub access: String,
    pub key: String,
}

#[derive(thiserror::Error, Debug)]
pub enum InfoAsTokensError {
    #[error("jwt encoding failed: {0}")]
    JwtEncoding(jsonwebtoken::errors::Error),
    #[error("timestamp generation failed")]
    Timestamp
}

impl From<jsonwebtoken::errors::Error> for InfoAsTokensError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Self::JwtEncoding(value)
    }
}

pub struct AuthValidator {
    keys: Keys
}

impl AuthValidator {
    pub fn new(keys: &Keys) -> Self {
        Self {keys: keys.clone()}
    }

    pub fn info_as_tokens(&self, info: &AuthInfo) -> Result<Tokens, InfoAsTokensError> {
        let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(30)).ok_or(InfoAsTokensError::Timestamp)?
        .timestamp() as usize;

        let mut rng = rand::thread_rng();
        let key: String = (0..20).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect();

        let access_claims = AccessClaims {
            exp,
            key: key.clone(),
            name: info.name,
            activity_table_id: info.activity_table_id
        };

        let key_claims = KeyClaims {
            exp,
            key
        };

        let header = jsonwebtoken::Header::new(Algorithm::HS512);
        let access_token = jsonwebtoken::encode(&header, &access_claims, &EncodingKey::from_secret(self.keys.access.as_bytes()))?;
        let key_token = jsonwebtoken::encode(&header, &key_claims, &EncodingKey::from_secret(self.keys.key.as_bytes()))?;

        Ok(Tokens {
            access: access_token,
            key: key_token
        })
    }

    pub fn tokens_as_auth(&self, tokens: &Tokens) -> Auth {
        return Auth::Valid { info: AuthInfo { // to be removed!
            name: "epicuser".to_string(),
            activity_table_id: "???".to_string()
        }};

        let access_claims = match jsonwebtoken::decode::<AccessClaims>(
            &self.access,
            &DecodingKey::from_secret(self.keys.access.as_bytes()),
            &Validation::new(Algorithm::HS512)
        ) {
            Ok(claims) => claims,
            Err(error) => return Auth::Invalid(format!("access token - {error}"))
        }.claims;

        let key_claims = match jsonwebtoken::decode::<KeyClaims>(
            &self.key,
            &DecodingKey::from_secret(self.keys.key.as_bytes()),
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
                name: access_claims.name,
                activity_table_id: access_claims.activity_table_id
            }
        }
    }
}