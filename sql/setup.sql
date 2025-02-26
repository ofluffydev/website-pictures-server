-- Table to store service sections
CREATE TABLE IF NOT EXISTS ServiceSections (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Table to store services under each section
CREATE TABLE IF NOT EXISTS Services (
    id SERIAL PRIMARY KEY,
    section_id INT NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (section_id) REFERENCES ServiceSections(id) ON DELETE CASCADE
);