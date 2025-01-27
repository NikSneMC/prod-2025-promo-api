SELECT count(*)
FROM promos
WHERE ((target).country IS NULL OR lower((target).country) = lower($1))
  AND lower($2) = ANY (lower((target).categories::text)::text[])
  and promos.active = $3
