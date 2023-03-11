use std::collections::BTreeMap;
use surrealdb::sql::{Object, Value};
use super::{DbPool, Error};
use super::utils::{b_tree_map, extract_single_object, query_result_into_objects, query_db};

pub struct User {
    pub name: String,
    pub email: String,
    pub password_hash: String,
}

impl TryFrom<Object> for User {
    type Error = String;
    
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        Ok(Self {
            name: object.get("name").ok_or("Name not found")?.clone().as_string(),
            email: object.get("email").ok_or("Email not found")?.clone().as_string(),
            password_hash: object.get("password_hash").ok_or("Password hash not found")?.clone().as_string()
        })
    }
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
        let vars = b_tree_map!(
            ("name", name)
        );

        let responses = query_db!(self, "SELECT * FROM user WHERE name = $name", vars);
        let objects = query_result_into_objects(responses)?;
        User::try_from(extract_single_object(objects)?).map_err(|error| Error::Conversion(error))
    }

    pub async fn create_user(&self, name: &str, email: &str, password_hash: &str) -> Result<(), Error> {
        let values: BTreeMap<String, Value> = b_tree_map!(
            ("name", name),
            ("email", email),
            ("password_hash", password_hash)
        );
        let vars = b_tree_map!(
            ("values", values)
        );

        query_db!(self, "CREATE user CONTENT $values", vars);
        Ok(())
    }

    
    pub async fn check_if_unique_credentials(&self, name: &str, email: &str) -> Result<CredentialUniqueness, Error> {
        let vars: BTreeMap<String, Value> = b_tree_map!(
            ("name", name.to_string()),
            ("email", email.to_string())
        );

        let responses = query_db!(self, "SELECT name, email FROM user WHERE name = $name OR email = $email", vars);
        let mut objects = query_result_into_objects(responses)?;
        
        match objects.next() {
            Some(object_result) => match object_result {
                Ok(object) => Ok(CredentialUniqueness {
                    name: object.get("name").ok_or(Error::Conversion("name field doesn't exist".to_owned()))?.to_string().as_str() != name,
                    email: object.get("email").ok_or(Error::Conversion("email field doesn't exist".to_owned()))?.to_string().as_str() != email
                }),
                Err(error) => Err(Error::ObjectFailure(error))
            }
            None => Ok(CredentialUniqueness::default())
        }
    }
}