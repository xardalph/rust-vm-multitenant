-- Create company table
create table if not exists company
(
    id integer primary key not null,
    name text not null unique
);
-- Create users table.
create table if not exists users
(
    id integer primary key not null,
    username text not null unique,
    password text not null,
    id_company integer not null,
    FOREIGN KEY (id_company) REFERENCES company(id)
);

create table if not exists agent
(
    id integer primary key not null,
    name text not null unique,
    token text not null,
    id_company integer not null,
    FOREIGN KEY (id_company) REFERENCES company(id)
);

-- Insert first company with a user and one agent.

insert into company (id, name)
values(1,"ensiie");

insert into users (id, username, password, id_company)
values (1, 'ferris', '$argon2id$v=19$m=19456,t=2,p=1$VE0e3g7DalWHgDwou3nuRA$uC6TER156UQpk0lNQ5+jHM0l5poVjPA1he/Tyn9J4Zw', 1);

insert into agent(id, name, token, id_company)
values(1, "first agent", "secrettokenreversible", 1);
