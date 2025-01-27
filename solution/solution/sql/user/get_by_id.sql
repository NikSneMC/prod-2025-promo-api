SELECT id, name, surname, email, avatar_url, other AS "other: DBUserTargetSettings", password_hash
FROM users
WHERE id = $1