SELECT count(*)
FROM promos
WHERE ((target).country IS NULL OR lower((target).country) = lower($1))
  AND $2 = promos.active
