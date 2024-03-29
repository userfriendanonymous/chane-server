use serde::{Serialize, Deserialize};
use ts_rs::TS;
use super::{Error, DbPool, utils::as_obj_id};
use mongodb::bson::{doc, oid::ObjectId};

#[derive(Serialize, Deserialize, Debug)]
pub struct Role {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub owner: String,
    pub editors: Vec<String>,
    pub name: String,
    pub extends: Vec<String>,
    pub permissions: RolePermissions
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, TS)]
#[ts(export)]
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
    pub live: Vec<String>,
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
        append_vec_unique(&mut self.live, &external.live);
        self.set_labels = self.set_labels || external.set_labels;
    }
}

impl DbPool {
    pub async fn get_role(&self, id: &str) -> Result<Role, Error> {
        let result = self.roles.find_one(doc! {"_id": as_obj_id(id)?}, None).await?;
        match result {
            Some(model) => Ok(model),
            None => Err(Error::NotFound)
        }
    }

    pub async fn create_role(&self, name: &str, owner: &str, extends: &[String], editors: &[String], permissions: &RolePermissions) -> Result<String, Error> {
        let model = Role {
            id: None,
            owner: owner.to_owned(),
            editors: editors.to_owned(),
            extends: extends.to_owned(),
            name: name.to_owned(),
            permissions: permissions.clone()
        };
        let result = self.roles.insert_one(model, None).await?;
        Ok(result.inserted_id.to_string())
    }

    pub async fn change_role(
        &self,
        id: &str,
        name: &str,
        extends: &[String],
        editors: &Option<Vec<String>>,
        permissions: RolePermissions // NOT &
    ) -> Result<(), Error> {
        let result = self.roles.update_one(doc! {"id": id}, doc! {"$set": {
            "name": name,
            "extends": extends,
            "editors": editors,
            "permissions": {
                "change_roles": permissions.change_roles,
                "view_blocks": permissions.view_blocks,
                "connect_blocks": permissions.connect_blocks,
                "disconnect_blocks": permissions.disconnect_blocks,
                "pin_block": permissions.pin_block,
                "change_default_role": permissions.change_default_role,
                "change_description": permissions.change_description,
                "pin_roles": permissions.pin_roles,
                "set_labels": permissions.set_labels,
                "live": permissions.live,
            }
        }}, None).await?;
        if result.modified_count == 0 {
            return Err(Error::NotFound)
        }
        Ok(())
    }
}