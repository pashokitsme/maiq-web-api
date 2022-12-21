drop table if exists groups;

create table groups (
    uid serial primary key,
    name varchar(16) not null
);

drop table if exists snapshots;

create table snapshots (
    uid varchar(128) unique not null,
    date date not null,
    parsed_at timestamp not null default localtimestamp
);

drop table if exists lessons;

create table lessons (
    uid serial primary key,
    group_id integer not null,
    day_id integer not null,
    name varchar(256) not null,
    num integer not null,
    subgroup integer default null,
    teacher varchar(128) default null,
    classroom varchar(128) default null
);