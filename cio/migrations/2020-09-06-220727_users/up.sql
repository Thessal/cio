CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    username VARCHAR NOT NULL UNIQUE,
    aliases TEXT [] NOT NULL,
    recovery_email VARCHAR NOT NULL,
    recovery_phone VARCHAR NOT NULL,
    gender VARCHAR NOT NULL,
    chat VARCHAR NOT NULL,
    github VARCHAR NOT NULL,
    twitter VARCHAR NOT NULL,
    groups TEXT [] NOT NULL,
    is_group_admin BOOLEAN NOT NULL DEFAULT 'f',
    is_system_account BOOLEAN NOT NULL DEFAULT 'f',
    building VARCHAR NOT NULL,
    link_to_building TEXT [] NOT NULL,
    aws_role VARCHAR NOT NULL,
    home_address_street_1 VARCHAR NOT NULL,
    home_address_street_2 VARCHAR NOT NULL,
    home_address_city VARCHAR NOT NULL,
    home_address_state VARCHAR NOT NULL,
    home_address_zipcode VARCHAR NOT NULL,
    home_address_country VARCHAR NOT NULL,
    home_address_formatted VARCHAR NOT NULL,
    start_date DATE NOT NULL,
    birthday DATE NOT NULL,
    public_ssh_keys [] TEXT NOT NULL
)
