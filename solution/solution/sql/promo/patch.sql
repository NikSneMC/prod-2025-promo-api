WITH updated_promo AS (
    UPDATE promos
        SET description = coalesce($2, description),
            image_url = coalesce($3, image_url),
            target = coalesce($4, target),
            max_count = coalesce($5, max_count),
            active_from = coalesce($6, active_from),
            active_until = coalesce($7, active_until)
        WHERE id = $1
        RETURNING *)
SELECT updated_promo.id,
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
FROM updated_promo
         LEFT JOIN companies ON companies.id = company_id