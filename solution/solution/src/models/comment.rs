use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::database::models::DBComment;

use super::User;

#[derive(Deserialize, Validate, Debug)]
pub struct CommentPath {
    pub promo_id: Uuid,
    pub comment_id: Uuid,
}

#[derive(Serialize, Debug)]
pub struct CommentAuthor {
    pub name: String,

    pub surname: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

impl From<User> for CommentAuthor {
    fn from(user: User) -> Self {
        Self {
            name: user.name,
            surname: user.surname,
            avatar_url: user.avatar_url,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Comment {
    pub id: Uuid,

    #[serde(skip)]
    pub author_id: Uuid,

    #[serde(skip)]
    pub promo_id: Uuid,

    pub text: String,

    pub date: DateTime<Utc>,

    pub author: CommentAuthor,
}

impl Comment {
    pub fn into_db(self) -> DBComment {
        DBComment::from(self)
    }
}

impl From<DBComment> for Comment {
    fn from(db_comment: DBComment) -> Self {
        Self {
            id: db_comment.id,
            author_id: db_comment.author_id,
            promo_id: db_comment.promo_id,
            text: db_comment.text,
            date: db_comment.date,
            author: CommentAuthor {
                name: db_comment.author_name.unwrap(),
                surname: db_comment.author_surname.unwrap(),
                avatar_url: db_comment.author_avatar_url,
            },
        }
    }
}
