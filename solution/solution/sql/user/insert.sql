INSERT INTO users (id, name, surname, email, avatar_url, other, password_hash)
VALUES ($1, $2, $3, $4, $5, $6, $7)
RETURNING id, name, surname, email, avatar_url, other AS "other: DBUserTargetSettings", password_hash