SELECT promos.id,
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
FROM promos
         LEFT JOIN companies ON companies.id = company_id
WHERE ((target).country IS NULL OR lower((target).country) = lower($1))
  AND $2 = promos.active
ORDER BY id DESC
LIMIT $3 OFFSET $4
