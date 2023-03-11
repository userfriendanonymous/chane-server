use super::{Instance, Error as GeneralError, extract_auth, extract_db};

struct Channel {
    id: u64
}

#[derive(thiserror::Error, Debug)]
pub enum CreateError {
    #[error("general: {0}")]
    General(GeneralError)
}

impl Instance {
    pub async fn create_channel(&self) -> Result<u64, CreateError> {
        let auth = extract_auth!(self, CreateError::General);
        extract_db!(self, db_pool, db_pool_cloned);

        let id = db_pool.create_channel().await.map_err(|error| CreateError::General(GeneralError::Db(error)))?;
        Ok(id)
    }
}