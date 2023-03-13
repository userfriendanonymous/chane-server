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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RolePermissions {
    pub change_roles: Vec<String>,
    pub view_blocks: Vec<String>,
    pub connect_blocks: Vec<String>,
    pub disconnect_blocks: Vec<String>,
    pub pin_block: Vec<String>,
    pub change_default_role: Vec<String>,
    pub change_description: Vec<String>,
    pub pin_roles: Vec<String>,
    pub set_labels: bool,
}

fn append_vec_unique<V: PartialEq + Clone>(vec1: &mut Vec<V>, vec2: &Vec<V>){
    for item in vec2 {
        if !vec1.contains(item){
            vec1.push(item.clone())
        }
    }
}

impl RolePermissions {
    pub fn add(&mut self, external: &RolePermissions){
        append_vec_unique(&mut self.change_roles, &external.change_roles);
        append_vec_unique(&mut self.view_blocks, &external.view_blocks);
        append_vec_unique(&mut self.connect_blocks, &external.connect_blocks);
        append_vec_unique(&mut self.disconnect_blocks, &external.disconnect_blocks);
        append_vec_unique(&mut self.pin_block, &external.pin_block);
        append_vec_unique(&mut self.change_default_role, &external.change_default_role);
        append_vec_unique(&mut self.change_description, &external.change_description);
        append_vec_unique(&mut self.pin_roles, &external.pin_roles);
        self.set_labels = self.set_labels || external.set_labels;
    }
}

impl DbPool {
    pub async fn get_role(&self, id: &str) -> Result<Role, Error> {
        let result = self.roles.find_one(doc! {"_id": as_object_id!(id)}, None).await?;
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
        let result = self.roles.insert_one(model, None).await?;
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
                "pin_block": permissions.pin_block.clone(),
                "change_default_role": permissions.change_default_role.clone(),
                "change_description": permissions.change_description.clone(),
                "pin_roles": permissions.pin_roles.clone(),
                "set_labels": permissions.set_labels.clone()
            }
        }}, None).await?;
        Ok(())
    }
}