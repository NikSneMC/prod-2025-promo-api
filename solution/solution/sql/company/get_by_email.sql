SELECT id, name, email, password_hash
FROM companies
WHERE email = lower($1)