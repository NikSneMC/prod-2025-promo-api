WITH inserted_comment AS (
    INSERT INTO comments (id, author_id, promo_id, text, date)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *)
SELECT inserted_comment.id,
       author_id,
       promo_id,
       text,
       date,
       users.name       AS author_name,
       users.surname    AS author_surname,
       users.avatar_url AS author_avatar_url
FROM inserted_comment
         LEFT JOIN users ON users.id = author_id