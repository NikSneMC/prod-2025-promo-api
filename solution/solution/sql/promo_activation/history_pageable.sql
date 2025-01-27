WITH activations AS (SELECT promo_id,
                            company_id,
                            description,
                            image_url,
                            target,
                            max_count,
                            active_from,
                            active_until,
                            mode,
                            promo_common,
                            promo_unique,
                            like_count,
                            used_count,
                            comment_count,
                            active
                     FROM activations
                              LEFT JOIN promos ON promos.id = activations.promo_id
                     WHERE user_id = $1
                     ORDER BY date DESC
                     LIMIT $2 OFFSET $3)
SELECT promo_id as id,
       company_id,
       companies.name AS company_name,
       description,
       image_url,
       target         AS "target: DBTarget",
       max_count,
       active_from,
       active_until,
       mode           AS "mode: DBPromoMode",
       promo_common,
       promo_unique,
       like_count,
       used_count,
       comment_count,
       active
FROM activations
         LEFT JOIN companies ON companies.id = company_id
ORDER BY id DESC
