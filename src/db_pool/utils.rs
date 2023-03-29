use mongodb::bson::oid::ObjectId;

pub fn as_obj_id(id: &str) -> mongodb::bson::oid::Result<ObjectId> {
    ObjectId::parse_str(id)
}