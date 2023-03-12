use serde::{Serialize, Deserialize};
use super::{DbPool, Error};
use mongodb::bson::doc;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub groups: Vec<String>
}

pub struct CredentialUniqueness {
    pub name: bool,
    pub email: bool
}

impl Default for CredentialUniqueness {
    fn default() -> Self {
        Self {
            name: true,
            email: true
        }
    }
}

impl DbPool {
    pub async fn get_user(&self, name: &str) -> Result<User, Error> {
        let filter = doc! {"name": name};
        let result = self.users.find_one(filter, None).await.map_err(Error::Query)?;
        match result {
            Some(model) => Ok(model),
            None => Err(Error::NotFound)
        }
    }

    pub async fn create_user(&self, name: &str, email: &str, password_hash: &str) -> Result<(), Error> {
        let document = User {
            email: email.to_string(),
            name: name.to_string(),
            password_hash: password_hash.to_string(),
            groups: Vec::new()
        };
        self.users.insert_one(document, None).await.map_err(Error::Query)?;
        Ok(())
    }

    pub async fn check_if_unique_credentials(&self, name: &str, email: &str) -> Result<CredentialUniqueness, Error> {
        let filter = doc! {"$or": [{"name": name}, {"email": email}]};
        let result = self.users.find_one(filter, None).await.map_err(Error::Query)?;
        Ok(match result {
            None => CredentialUniqueness::default(),
            Some(model) => {
                CredentialUniqueness {
                    email: model.email.as_str() != email,
                    name: model.name.as_str() != name,
                }
            }
        })
    }
}