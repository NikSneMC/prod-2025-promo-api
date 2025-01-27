use crate::database::models::DBCompany;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password_hash: String,
}

impl Company {
    pub fn into_db(self) -> DBCompany {
        DBCompany::from(self)
    }
}

impl From<DBCompany> for Company {
    fn from(db_company: DBCompany) -> Self {
        Self {
            id: db_company.id,
            name: db_company.name,
            email: db_company.email.to_string(),
            password_hash: db_company.password_hash,
        }
    }
}
