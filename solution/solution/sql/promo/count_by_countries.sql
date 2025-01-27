SELECT count(*)
FROM promos
WHERE company_id = $1
  AND ((target).country IS NULL
    OR lower((target).country) = ANY ($2))
