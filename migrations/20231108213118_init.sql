-- Create company table
create table if not exists company
(
    id SERIAL primary key,
    name text not null unique
);
ALTER SEQUENCE company_id_seq MINVALUE 0
START 0
RESTART 0;

-- Create users table.
create table if not exists users
(
    id SERIAL primary key ,
    username text not null unique,
    password text not null,
    id_company integer not null,
    FOREIGN KEY (id_company) REFERENCES company(id)
);

create table if not exists agent
(
    id SERIAL primary key ,
    name text not null unique,
    token text not null,
    id_company integer not null,
    FOREIGN KEY (id_company) REFERENCES company(id)
);

-- Insert first company with a user and one agent.

insert into company (name)
values('ensiie');

insert into company (name)
values('tsp');

insert into users ( username, password, id_company)
values ('admin', '$argon2id$v=19$m=19456,t=2,p=1$VE0e3g7DalWHgDwou3nuRA$uC6TER156UQpk0lNQ5+jHM0l5poVjPA1he/Tyn9J4Zw', 0);

insert into agent( name, token, id_company)
values('first Ensiie agent', 'secrettokenreversible', 0);
insert into agent( name, token, id_company)
values('tps agent', 'secrettokenreversible', 0);
insert into agent( name, token, id_company)
values('second Ensiie agent', 'secrettokenreversible2', 1);
