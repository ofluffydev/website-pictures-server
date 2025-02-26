INSERT INTO Services (section_id, name, description)
VALUES ($1, $2, $3)
RETURNING *;
