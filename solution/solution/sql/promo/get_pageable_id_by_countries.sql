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
WHERE company_id = $1
  AND ((target).country IS NULL
    OR lower((target).country) = ANY (lower(text($2::text[]))::text[]))
ORDER BY id DESC
LIMIT $3 OFFSET $4
