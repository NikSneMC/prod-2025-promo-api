INSERT INTO companies (id, name, email, password_hash)
VALUES ($1, $2, $3, $4)
RETURNING id, name, email, password_hash