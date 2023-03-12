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
    pub change_roles: Vec<String>,
    pub view_blocks: Vec<String>,
    pub add_blocks: Vec<String>,
    pub remove_blocks: Vec<String>,
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

    pub async fn create_role(
        &self,
        name: String,
        owner: String,
        extends: Vec<String>,
        editors: Vec<String>,
        change_roles: Vec<String>,
        view_blocks: Vec<String>,
        add_blocks: Vec<String>,
        remove_blocks: Vec<String>,
        pin_blocks: Vec<String>,
        change_default_role: Vec<String>,
        change_description: Vec<String>,
        pin_roles: Vec<String>,
    ) -> Result<String, Error> {
        let model = Role {
            id: None,
            owner,
            add_blocks,
            change_default_role,
            change_description,
            change_roles,
            editors,
            extends,
            name,
            pin_blocks,
            pin_roles,
            remove_blocks,
            view_blocks
        };
        let result = self.roles.insert_one(model, None).await.map_err(Error::Query)?;
        Ok(result.inserted_id.to_string())
    }

    pub async fn update_role(
        &self,
        id: String,
        name: String,
        extends: Vec<String>,
        editors: Option<Vec<String>>,
        change_roles: Vec<String>,
        view_blocks: Vec<String>,
        add_blocks: Vec<String>,
        remove_blocks: Vec<String>,
        pin_blocks: Vec<String>,
        change_default_role: Vec<String>,
        change_description: Vec<String>,
        pin_roles: Vec<String>,
    ) -> Result<(), Error> {
        let result = self.roles.update_one(doc! {"id": id}, doc! {"$set": {
            "name": name,
            "extends": extends,
            "editors": editors,
            "change_roles": change_roles,
            "view_blocks": view_blocks,
            "add_blocks": add_blocks,
            "remove_blocks": remove_blocks,
            "pin_blocks": pin_blocks,
            "change_defualt_role": change_default_role,
            "change_description": change_description,
            "pin_roles": pin_roles,
        }}, None).await.map_err(Error::Query)?;
        Ok(())
    }
}