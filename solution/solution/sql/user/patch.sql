UPDATE users
SET name          = COALESCE($2, name),
    surname       = COALESCE($3, surname),
    avatar_url    = COALESCE($4, avatar_url),
    password_hash = COALESCE($5, password_hash)
WHERE id = $1
RETURNING id, name, surname, email, avatar_url, other AS "other: DBUserTargetSettings", password_hash