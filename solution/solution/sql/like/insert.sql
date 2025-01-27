INSERT
INTO likes (user_id, promo_id)
VALUES ($1, $2)
RETURNING *