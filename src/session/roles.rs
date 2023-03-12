use serde::{Serialize, Deserialize};
use crate::db_pool::{self, RolePermissions, DbPoolGuard, Channel};
use super::{Error as GeneralError, extract_db, extract_auth, Session};

#[derive(Serialize, Deserialize)]
struct Role {
    pub id: String,
    pub owner: String,
    pub editors: Vec<String>,
    pub name: String,
    pub extends: Vec<String>,
    pub permissions: RolePermissions
}

impl From<db_pool::Role> for Role {
    fn from(role: db_pool::Role) -> Self {
        Self {
            id: role.id.unwrap(),
            name: role.name,
            owner: role.owner,
            editors: role.editors,
            extends: role.extends,
            permissions: role.permissions
        }
    }
}

pub async fn get_user_role<'g>(db_pool: &DbPoolGuard<'g>, channel_id: &str, user_name: &str) -> Result<(Role, Channel), GeneralError> {
    let channel = db_pool.get_channel(channel_id).await.map_err(GeneralError::Db)?;

    let role_id = &channel.default_role;
    for role in &channel.roles {
        if role.0 == user_name {
            let role_id = &role.1;
            break;
        }
    }
    Ok((Role::from(db_pool.get_role(role_id).await.map_err(GeneralError::Db)?), channel))
}

fn catch_vec_intersection<T: PartialEq>(vec1: &Vec<T>, vec2: &Vec<T>) -> bool {
    for item in vec1 {
        if vec2.contains(&item) {
            return true
        }
    }
    false
}

pub struct RolePermissionValidator<'a> {
    labels: &'a Vec<String>,
    permissions: &'a RolePermissions
}


impl<'a> RolePermissionValidator<'a> {
    pub fn new(permissions: &'a RolePermissions, labels: &'a Vec<String>) -> Self {
        Self {
            labels,
            permissions
        }
    }
    pub fn can_connect_blocks(&self) -> bool {
        catch_vec_intersection(self.labels, &self.permissions.connect_blocks)
    }
    pub fn can_disconnect_blocks(&self) -> bool {
        catch_vec_intersection(self.labels, &self.permissions.disconnect_blocks)
    }
}

impl Session {
    pub async fn get_role(&self, id: &str) -> Result<Role, GeneralError> {
        extract_db!(self, db_pool, db_pool_cloned);
        Ok(Role::from(db_pool.get_role(id).await.map_err(GeneralError::Db)?))
    }

    pub async fn create_role(&self, name: &String, owner: &String, extends: &Vec<String>, editors: &Vec<String>, permissions: &RolePermissions) -> Result<String, GeneralError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);
        db_pool.create_role(name, owner, extends, editors, permissions).await.map_err(GeneralError::Db)
    }

    pub async fn update_role(&self, id: &String, name: &String, extends: &Vec<String>, editors: &Vec<String>, permissions: &RolePermissions) -> Result<(), GeneralError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);
        let role = db_pool.get_role(id).await.map_err(GeneralError::Db)?;

        let editors = if role.owner == auth.name {
            Some(editors.clone())
        } else if role.editors.contains(&auth.name) {
            None
        } else {
            return Err(GeneralError::Unauthorized("You don't have permissions to edit this role".to_owned()))
        };
        
        db_pool.update_role(id, name, extends, &editors, permissions).await.map_err(GeneralError::Db)
    }
}