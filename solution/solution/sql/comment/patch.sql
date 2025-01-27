WITH updated_comment AS (
    UPDATE comments
        SET
            text = $2
        WHERE id = $1
        RETURNING *)
SELECT updated_comment.id,
       author_id,
       promo_id,
       text,
       date,
       users.name       AS author_name,
       users.surname    AS author_surname,
       users.avatar_url AS author_avatar_url
FROM updated_comment
         LEFT JOIN users ON users.id = author_id