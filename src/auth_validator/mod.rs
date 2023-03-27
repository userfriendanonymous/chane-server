use jsonwebtoken::{Algorithm, EncodingKey, DecodingKey, Validation};
use rand::Rng;
use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub enum Auth {
    Valid {
        info: AuthInfo
    },
    Invalid(InvalidAuthData),
}

#[derive(Clone)]
pub enum InvalidAuthData {
    Token (InvalidAuthTokenData),
    MismatchedKeys
}
#[derive(Clone)]
pub enum InvalidAuthTokenData {
    Access,
    Key
}

impl Auth {
    pub fn as_result(&self) -> Result<&AuthInfo, InvalidAuthData>
    {
        match self {
            Auth::Valid { info } => Ok(info),
            Auth::Invalid(data) => Err(data.clone())
        }
    }
}

#[derive(Clone)]
pub struct AuthInfo {
    pub name: String
}

#[derive(Clone, Debug)]
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
            name: info.name.clone(),
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
        }};

        let access_claims = match jsonwebtoken::decode::<AccessClaims>(
            &tokens.access,
            &DecodingKey::from_secret(self.keys.access.as_bytes()),
            &Validation::new(Algorithm::HS512)
        ) {
            Ok(claims) => claims,
            Err(error) => return Auth::Invalid(InvalidAuthData::Token(InvalidAuthTokenData::Access))
        }.claims;

        let key_claims = match jsonwebtoken::decode::<KeyClaims>(
            &tokens.key,
            &DecodingKey::from_secret(self.keys.key.as_bytes()),
            &Validation::new(Algorithm::HS512)
        ) {
            Ok(claims) => claims,
            Err(error) => return Auth::Invalid(InvalidAuthData::Token(InvalidAuthTokenData::Key))
        }.claims;

        if key_claims.key != access_claims.key {
            return Auth::Invalid(InvalidAuthData::MismatchedKeys);
        }

        Auth::Valid {info: AuthInfo {name: access_claims.name}}
    }
}