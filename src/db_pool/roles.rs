use serde::{Serialize, Deserialize};
use super::{Error, DbPool, utils::as_object_id};
use mongodb::bson::{doc, oid::ObjectId};

#[derive(Serialize, Deserialize, Debug)]
pub struct Role {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub owner: String,
    pub editors: Vec<String>,
    pub name: String,
    pub extends: Vec<String>,
    pub permissions: RolePermissions
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RolePermissions {
    pub change_roles: Vec<String>,
    pub view_blocks: Vec<String>,
    pub connect_blocks: Vec<String>,
    pub disconnect_blocks: Vec<String>,
    pub pin_blocks: Vec<String>,
    pub change_default_role: Vec<String>,
    pub change_description: Vec<String>,
    pub pin_roles: Vec<String>,
}

impl DbPool {
    pub async fn get_role(&self, id: &str) -> Result<Role, Error> {
        let result = self.roles.find_one(doc! {"_id": as_object_id!(id)}, None).await.map_err(Error::Query)?;
        match result {
            Some(model) => Ok(model),
            None => Err(Error::NotFound)
        }
    }

    pub async fn create_role(&self, name: &String, owner: &String, extends: &Vec<String>, editors: &Vec<String>, permissions: &RolePermissions) -> Result<String, Error> {
        let model = Role {
            id: None,
            owner: owner.clone(),
            editors: editors.clone(),
            extends: extends.clone(),
            name: name.clone(),
            permissions: permissions.clone()
        };
        let result = self.roles.insert_one(model, None).await.map_err(Error::Query)?;
        Ok(result.inserted_id.to_string())
    }

    pub async fn update_role(
        &self,
        id: &String,
        name: &String,
        extends: &Vec<String>,
        editors: &Option<Vec<String>>,
        permissions: &RolePermissions
    ) -> Result<(), Error> {
        let result = self.roles.update_one(doc! {"id": id}, doc! {"$set": {
            "name": name,
            "extends": extends,
            "editors": editors,
            "permissions": {
                "change_roles": permissions.change_roles.clone(),
                "view_blocks": permissions.view_blocks.clone(),
                "connect_blocks": permissions.connect_blocks.clone(),
                "disconnect_blocks": permissions.disconnect_blocks.clone(),
                "pin_blocks": permissions.pin_blocks.clone(),
                "change_default_role": permissions.change_default_role.clone(),
                "change_description": permissions.change_description.clone(),
                "pin_roles": permissions.pin_roles.clone()
            }
        }}, None).await.map_err(Error::Query)?;
        Ok(())
    }
}