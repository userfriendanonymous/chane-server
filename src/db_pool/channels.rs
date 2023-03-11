use std::collections::BTreeMap;
use surrealdb::sql::{Object, Value};
use super::{DbPool, Error};
use super::utils::{b_tree_map, extract_single_object, query_result_into_objects, query_db};

pub struct Channel {
    id: u64,
}

impl TryFrom<Object> for Channel {
    type Error = String;
    
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        Ok(Self {
            id: object.get("id").ok_or("Id not found")?.clone().as_int() as u64,
        })
    }
}

impl DbPool {
    pub async fn get_channel(&self, id: &u64) -> Result<Channel, Error> {
        let vars = b_tree_map!(
            ("id", id.clone())
        );

        let responses = query_db!(self, "SELECT * FROM channel WHERE id = $id", vars);
        let objects = query_result_into_objects(responses)?;
        Channel::try_from(extract_single_object(objects)?).map_err(|error| Error::Conversion(error))
    }

    pub async fn create_channel(&self) -> Result<u64, Error> {
        let values: BTreeMap<String, Value> = b_tree_map!(
        );
        let vars = b_tree_map!(
            ("values", values)
        );

        let responses = query_db!(self, "CREATE channel CONTENT $values", vars);
        let objects = query_result_into_objects(responses)?;
        let object = extract_single_object(objects)?;
        let id = object.get("id").ok_or(Error::Conversion("Id not found".to_owned()))?.clone().as_int() as u64;
        Ok(id)
    }
}