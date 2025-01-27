SELECT comments.id,
       author_id,
       promo_id,
       text,
       date,
       users.name       AS author_name,
       users.surname    AS author_surname,
       users.avatar_url AS author_avatar_url
FROM comments
         LEFT JOIN users ON users.id = author_id
WHERE promo_id = $1
ORDER BY date DESC
LIMIT $2 OFFSET $3