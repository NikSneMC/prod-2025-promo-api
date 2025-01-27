WITH common_promo AS (
    UPDATE promos
        SET used_count = used_count + 1
        WHERE id = $2
        RETURNING promos.promo_common AS promo
)
INSERT INTO activations (user_id, promo_id, promo, date)
VALUES ($1, $2, (SELECT promo FROM common_promo), $3)
RETURNING *