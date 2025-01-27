SELECT user_id,
       promo_id,
       promo,
       date
FROM activations
WHERE user_id = $1
  AND promo_id = $2
