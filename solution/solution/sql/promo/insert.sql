WITH inserted_promo AS (
    INSERT INTO promos (id, company_id, description, image_url, target, max_count, active_from, active_until, mode,
                        promo_common, promo_unique, like_count, used_count, comment_count, active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING *)
SELECT inserted_promo.id,
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
FROM inserted_promo
         LEFT JOIN companies ON companies.id = company_id