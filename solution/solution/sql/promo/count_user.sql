SELECT count(*)
FROM promos
WHERE (target).country IS NULL
   OR lower((target).country) = lower($1)