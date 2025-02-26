SELECT id, section_id, name, description, created_at
FROM Services
WHERE section_id = $1;
