SELECT lower((users.other).country) as country, count(*) as activations_count
FROM activations
         LEFT JOIN users ON users.id = user_id
WHERE promo_id = $1
GROUP BY country
ORDER BY country
