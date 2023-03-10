use surrealdb::{sql::{self, Value, Object}, Response};
use super::Error;

macro_rules! b_tree_map {
    (
        $(($name:expr, $value:expr)),*
        $(,)? // for trailing commas
    ) => {
        [
            $(
                ($name.into(), $value.into()),
            )*
        ].into()
    };
}
pub(crate) use b_tree_map;

macro_rules! query_db {
    ($self:expr, $query:expr, $vars:expr) => {
        $self.datastore.execute(
            $query,
            &$self.session, Some($vars), true
        ).await.map_err(|error| Error::Query(error.to_string()))?
    };

    ($self:expr, $query:expr) => {
        $self.datastore.execute(
            $query,
            &$self.session, None($vars), true
        ).await.map_err(|error| Error::Query(error.to_string()))?
    };
}
pub(super) use query_db;

pub fn query_result_into_objects(responses: Vec<Response>) -> Result<impl Iterator<Item = Result<Object, String>>, Error> {
    let r = responses.into_iter().next().map(|response| response.result);

    let Some(Ok(Value::Array(sql::Array(values)))) = r else {
        return Err(Error::ObjectsNotFound);
    };

    Ok(values.into_iter().map(|value| match value {
        Value::Object(value) => Ok(value),
        error => Err(error.to_string())
    }))
}

pub fn extract_single_object(mut objects: impl Iterator<Item = Result<Object, String>>) -> Result<Object, Error> {
    match objects.next() {
        Some(object) => {
            object.map_err(|error| Error::ObjectFailure(error.to_string()))
        },

        None => Err(Error::FailedToExtractFirstObject)
    }
}