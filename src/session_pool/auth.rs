use crate::{db_pool, auth_validator::{self, Tokens, AuthInfo, InfoAsTokensError}};
use crate::activity_logger::Activity;
use super::{Session, Error as GeneralError};
use pwhash::bcrypt;
use serde::Serialize;
use ts_rs::TS;

const NAME_CHARS: &str = "QAZWSXEDCRFVTGBYHNUJMIKOLPqazwsxedcrfvtgbyhnujmikolp1234567890_";

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
    #[error("failed to convert info to tokens: {0}")]
    InfoAsTokens(InfoAsTokensError),
    #[error("failed to hash password: {0}")]
    Hashing(pwhash::error::Error)
}
impl From<db_pool::Error> for RegisterError {
    fn from(value: db_pool::Error) -> Self {
        Self::General(GeneralError::Db(value))
    }
}
impl From<InfoAsTokensError> for RegisterError {
    fn from(value: InfoAsTokensError) -> Self {
        Self::InfoAsTokens(value)
    }
}
impl From<pwhash::error::Error> for RegisterError {
    fn from(value: pwhash::error::Error) -> Self {
        Self::Hashing(value)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("failed to convert info to tokens: {0}")]
    InfoAsTokens(InfoAsTokensError),
    #[error("general error: {0}")]
    General(GeneralError),
    #[error("invalid credentials")]
    InvalidCredentials
}
impl From<db_pool::Error> for LoginError {
    fn from(value: db_pool::Error) -> Self {
        Self::General(GeneralError::Db(value))
    }
}
impl From<InfoAsTokensError> for LoginError {
    fn from(value: InfoAsTokensError) -> Self {
        Self::InfoAsTokens(value)
    }
}


#[derive(Serialize, TS)]
#[ts(export)]
#[serde(tag = "is", content = "data")]
pub enum AuthMe {
    Valid {
        name: String
    },
    Invalid
}
impl From<auth_validator::Auth> for AuthMe {
    fn from(auth: auth_validator::Auth) -> Self {
        match auth {
            auth_validator::Auth::Valid { ref info } => Self::Valid { name: info.name.clone() },
            auth_validator::Auth::Invalid(ref data) => Self::Invalid // WARNING "DATA" SHOULD BE USED (probably not silently ignored, instead should be logged somewhere!)
        }
    }
}


impl Session {
    pub async fn me(&self) -> AuthMe {
        self.auth.clone().into()
    }

    pub async fn login(&self, name: &str, password: &str) -> Result<Tokens, LoginError> {
        let user = self.db_pool.get_user(name).await?;
        if !bcrypt::verify(password, user.password_hash.as_str()) {
            return Err(LoginError::InvalidCredentials);
        }

        let tokens = self.auth_validator.info_as_tokens(&AuthInfo {
            name: name.to_string()
        })?;

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

        let uniqueness = self.db_pool.check_if_unique_credentials(name, email).await?;
        if !uniqueness.email {
            return Err(RegisterError::EmailTaken);
        }
        if !uniqueness.name {
            return Err(RegisterError::NameTaken);
        }

        let password_hash = bcrypt::hash(password)?;
        let tokens = self.auth_validator.info_as_tokens(&AuthInfo {
            name: name.to_string(),
        })?;

        let activity_table_id = self.db_pool.create_activity_table().await?;
        self.db_pool.create_user(name, email, password_hash.as_str(), &activity_table_id).await?;
        self.activity_logger.log(Activity::Joined { by: name.to_string() });

        Ok(tokens)
    }
}