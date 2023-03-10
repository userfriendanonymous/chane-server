use std::collections::BTreeMap;
use surrealdb::sql::{Object, Value};
use super::{DbPool, Error};
use super::utils::{b_tree_map, extract_single_object, query_result_into_objects, query_db};

pub struct Block {
    id: u64,
    content: String,
    author_name: String,
    channel_id: u64,
}

impl TryFrom<Object> for Block {
    type Error = String;
    
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        Ok(Self {
            id: object.get("id").ok_or("Id not found")?.clone().as_int() as u64,
            content: object.get("content").ok_or("Content not found")?.clone().as_string(),
            author_name: object.get("author_name").ok_or("Author name not found")?.clone().as_string(),
            channel_id: object.get("channel_id").ok_or("Channel id not found")?.clone().as_int() as u64,
        })
    }
}

impl DbPool {
    pub async fn get_block(&self, id: &u64) -> Result<Block, Error> {
        let vars = b_tree_map!(
            ("id", id.clone())
        );

        let responses = query_db!(self, "SELECT * FROM block WHERE id = $id", vars);
        let objects = query_result_into_objects(responses)?;
        Block::try_from(extract_single_object(objects)?).map_err(|error| Error::Conversion(error))
    }

    pub async fn create_block(&self, content: &str, author_name: &str, channel_id: &str) -> Result<(), Error> {
        let values: BTreeMap<String, Value> = b_tree_map!(
            ("content", content),
            ("author_name", author_name),
            ("channel_id", channel_id)
        );
        let vars = b_tree_map!(
            ("values", values)
        );

        query_db!(self, "CREATE block CONTENT $values", vars);
        Ok(())
    }
}