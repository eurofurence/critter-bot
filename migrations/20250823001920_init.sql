create table critters (
    id bigint not null primary key,
    tgid bigint not null unique
);

create table dates (
    "date" date not null primary key,
    notified boolean not null default false
);

create table shifts (
    id bigint not null primary key,
    start timestamp not null,
    stop timestamp not null,
    meta json not null
);
create index on shifts(start);

create table assignments (    
    shift bigint not null references shifts(id),
    critter bigint not null references critters(id),

    informed boolean not null default false,
    
    primary key(shift, critter)
);
create index on assignments(critter);

create table manager_assignments (    
    shift bigint not null references shifts(id),
    critter bigint not null references critters(id),

    informed boolean not null default false,
    
    primary key(shift, critter)
);
create index on manager_assignments(critter);
