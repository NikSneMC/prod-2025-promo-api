DELETE
FROM likes
WHERE user_id = $1
  AND promo_id = $2;
