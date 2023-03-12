macro_rules! as_object_id {
    ($id:expr) => {
        ObjectId::parse_str($id).map_err(Error::InvalidObjectId)?
    };
}
pub(super) use as_object_id;