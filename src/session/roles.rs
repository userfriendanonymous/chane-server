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

#[derive(thiserror::Error, Debug)]
pub enum CreateRoleError {
    #[error("general: {0}")]
    General(GeneralError),
    #[error("following role doesn't exist: {0}")]
    RoleDoesNotExist(String, db_pool::Error)
}
impl From<GeneralError> for CreateRoleError {
    fn from(value: GeneralError) -> Self {
        Self::General(value)
    }
}
impl From<db_pool::Error> for CreateRoleError {
    fn from(value: db_pool::Error) -> Self {
        Self::General(GeneralError::Db(value))
    }
}

impl Session {
    pub async fn get_role(&self, id: &str) -> Result<Role, GeneralError> {
        extract_db!(self, db_pool, db_pool_cloned);
        Ok(Role::from(db_pool.get_role(id).await?))
    }

    pub async fn create_role(&self, name: &String, extends: &Vec<String>, editors: &Vec<String>, permissions: &RolePermissions) -> Result<String, CreateRoleError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);

        for role_id in extends { // validates that all extending roles exist to avoid stuff like recursion to itself
            match db_pool.get_role(role_id).await {
                Ok(_) => {}
                Err(error) => return Err(CreateRoleError::RoleDoesNotExist(role_id.clone(), error))
            }
        }

        Ok(db_pool.create_role(name, &auth.name, extends, editors, permissions).await?)
    }

    pub async fn update_role(&self, id: &String, name: &String, extends: &Vec<String>, editors: &Vec<String>, permissions: &RolePermissions) -> Result<(), GeneralError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);
        let role = db_pool.get_role(id).await?;

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

#[derive(thiserror::Error, Debug)]
pub enum RoleWrappedError {
    #[error("Recursion detected at: {0}")]
    Recursion(String),
    #[error("General: {0}")]
    General(GeneralError),
}

impl From<GeneralError> for RoleWrappedError { // BRILLIANT! BYE, ".map_err(...)"
    fn from(value: GeneralError) -> Self {
        Self::General(value)
    }
}

impl From<db_pool::Error> for RoleWrappedError {
    fn from(value: db_pool::Error) -> Self {
        Self::General(GeneralError::Db(value))
    }
}

pub async fn resolve_user_role<'g>(db_pool: &DbPoolGuard<'g>, channel_id: &str, user_name: &str)
-> Result<(db_pool::Role, Channel, Vec<db_pool::Error>), RoleWrappedError>
{
    let channel = db_pool.get_channel(channel_id).await?;

    let mut role_id = &channel.default_role;
    for role in &channel.roles {
        if role.0 == user_name {
            role_id = &role.1;
            break;
        }
    }
    let (role, errors) = resolve_role(db_pool, role_id.as_str()).await?;
    Ok((role, channel, errors))
}

pub async fn resolve_role_permissions<'g>(db_pool: &DbPoolGuard<'g>, id: &str, permissions: &RolePermissions, extends: &Vec<String>)
-> Result<(RolePermissions, Vec<db_pool::Error>), RoleWrappedError>
{
    let mut permissions = permissions.clone();
    let mut role_ids = extends.clone();
    let mut processed_role_ids = role_ids.clone();
    processed_role_ids.push(id.to_string());

    let mut errors = Vec::new();

    while let Some(role_id) = role_ids.last() {
        let role = match db_pool.get_role(&role_id).await {
            Ok(role) => role,
            Err(error) => {
                errors.push(error);
                break;
            }
        };

        permissions.add(&role.permissions);

        for role_id in role.extends.iter() {
            if processed_role_ids.contains(role_id){
                return Err(RoleWrappedError::Recursion(role_id.to_string()));
            } else {
                role_ids.push(role_id.to_string());
                processed_role_ids.push(role_id.to_string());
            }
        }
    }

    Ok((permissions, errors))
}

pub async fn resolve_role<'g>(db_pool: &DbPoolGuard<'g>, id: &str) -> Result<(db_pool::Role, Vec<db_pool::Error>), RoleWrappedError> {
    let mut role = db_pool.get_role(&id).await?;
    let (permissions, errors) = resolve_role_permissions(db_pool, id, &role.permissions, &role.extends).await?;
    role.permissions = permissions;
    Ok((role, errors))
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
    pub fn can_pin_block(&self) -> bool {
        catch_vec_intersection(self.labels, &self.permissions.pin_block)
    }
    pub fn can_change_description(&self) -> bool {
        catch_vec_intersection(self.labels, &self.permissions.change_description)
    }
    pub fn can_change_default_role(&self) -> bool {
        catch_vec_intersection(self.labels, &self.permissions.change_default_role)
    }
    pub fn can_view_blocks(&self) -> bool {
        catch_vec_intersection(self.labels, &self.permissions.view_blocks)
    }
    pub fn can_pin_roles(&self) -> bool {
        catch_vec_intersection(self.labels, &self.permissions.pin_roles)
    }
    pub fn can_set_labels(&self) -> bool {
        self.permissions.set_labels
    }
}