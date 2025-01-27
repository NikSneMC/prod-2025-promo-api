use chrono::{DateTime, Utc};
use sqlx::{query_file, query_file_as, Executor, Postgres, Transaction};
use std::i64;
use uuid::Uuid;

use crate::models::Comment;

use super::DatabaseError;

#[derive(Debug)]
pub struct DBComment {
    pub id: Uuid,
    pub author_id: Uuid,
    pub promo_id: Uuid,
    pub text: String,
    pub date: DateTime<Utc>,
    pub author_name: Option<String>,
    pub author_surname: Option<String>,
    pub author_avatar_url: Option<String>,
}

impl DBComment {
    pub async fn insert(
        self,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Self, DatabaseError> {
        Ok(query_file_as!(
            Self,
            "sql/comment/insert.sql",
            self.id,
            self.author_id,
            self.promo_id,
            self.text,
            self.date
        )
        .fetch_one(&mut **transaction)
        .await?)
    }

    pub async fn get_pageable<'a, E>(
        promo_id: Uuid,
        limit: i64,
        offset: i64,
        executor: E,
    ) -> Result<(Vec<DBComment>, i64), DatabaseError>
    where
        E: Executor<'a, Database = Postgres> + Copy,
    {
        Ok((
            query_file_as!(
                Self,
                "sql/comment/get_pageable.sql",
                promo_id,
                limit,
                offset
            )
            .fetch_all(executor)
            .await?,
            query_file!("sql/comment/count.sql", promo_id)
                .fetch_one(executor)
                .await?
                .count
                .unwrap(),
        ))
    }

    pub async fn get_by_id<'a, E>(
        promo_id: Uuid,
        comment_id: Uuid,
        executor: E,
    ) -> Result<Option<Self>, DatabaseError>
    where
        E: Executor<'a, Database = Postgres>,
    {
        Ok(
            query_file_as!(Self, "sql/comment/get_by_id.sql", promo_id, comment_id)
                .fetch_optional(executor)
                .await?,
        )
    }

    pub async fn patch(
        self,
        text: String,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Self, DatabaseError> {
        Ok(query_file_as!(Self, "sql/comment/patch.sql", self.id, text)
            .fetch_one(&mut **transaction)
            .await?)
    }

    pub async fn delete(
        self,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), DatabaseError> {
        query_file!("sql/comment/delete.sql", self.id)
            .execute(&mut **transaction)
            .await?;

        Ok(())
    }

    pub fn into_model(self) -> Comment {
        Comment::from(self)
    }
}

impl From<Comment> for DBComment {
    fn from(comment: Comment) -> Self {
        Self {
            id: comment.id,
            author_id: comment.author_id,
            promo_id: comment.promo_id,
            text: comment.text,
            date: comment.date,
            author_name: Some(comment.author.name),
            author_surname: Some(comment.author.surname),
            author_avatar_url: comment.author.avatar_url,
        }
    }
}
