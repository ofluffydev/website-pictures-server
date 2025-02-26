INSERT INTO ServiceSections (title, description)
VALUES ($1, $2)
RETURNING *;
