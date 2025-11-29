-- Create company table
create table if not exists company
(
    id uuid DEFAULT uuidv7() primary key,
    name text not null unique,
    id_victoria SERIAL NOT NULL
);
ALTER SEQUENCE company_id_victoria_seq MINVALUE 0
START 0
RESTART 0;

-- Create users table.
create table if not exists users
(
    id uuid DEFAULT uuidv7() primary key,
    username text not null unique,
    password text not null,
    id_company uuid not null,
    FOREIGN KEY (id_company) REFERENCES company(id)
);

create table if not exists agent
(
    id uuid DEFAULT uuidv7() primary key,
    name text not null unique,
    token text not null unique,
    id_company uuid not null,
    FOREIGN KEY (id_company) REFERENCES company(id)
);

-- Insert first company with a user and one agent.

insert into company (name)
values('root');

insert into company (name)
values('ensiie');


INSERT INTO users ( username, password, id_company)
SELECT 'admin', '$argon2id$v=19$m=19456,t=2,p=1$VE0e3g7DalWHgDwou3nuRA$uC6TER156UQpk0lNQ5+jHM0l5poVjPA1he/Tyn9J4Zw', id FROM company where name = 'root';
INSERT INTO users ( username, password, id_company)
SELECT 'userEnsiie', '$argon2id$v=19$m=19456,t=2,p=1$VE0e3g7DalWHgDwou3nuRA$uC6TER156UQpk0lNQ5+jHM0l5poVjPA1he/Tyn9J4Zw', id FROM company where name = 'ensiie';

insert into agent( name, token, id_company)
SELECT 'main agent', 'mainAgentToken', id FROM company where name = 'root';
insert into agent( name, token, id_company)
SELECT 'Ensiie agent', 'EnsiieToken1', id FROM company where name = 'ensiie';
insert into agent( name, token, id_company)
SELECT 'second main agent', 'reversibleToken2ndAgent', id FROM company where name = 'root';
