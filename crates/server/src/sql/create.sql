CREATE TABLE users(
    uid INTEGER PRIMARY KEY AUTOINCREMENT,
    created_time INT NOT NULL,
    username TEXT NOT NULL UNIQUE,
    nickname TEXT,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    json TEXT
);
-- CREATE UNIQUE INDEX idx_users_username ON users(username)
CREATE UNIQUE INDEX idx_users_email ON users(email);
CREATE UNIQUE INDEX idx_users_username ON users(username);

CREATE TABLE tokens(
    token TEXT PRIMARY KEY,
    last_time INT NOT NULL,
    uid INT NOT NULL
);

CREATE INDEX idx_last_time ON tokens(last_time);

CREATE TABLE problems(
    pid TEXT PRIMARY KEY,
    owner INT,
    json TEXT
);

CREATE INDEX idx_problems_owner ON problems(owner);

CREATE TABLE records(
    rid INTEGER PRIMARY KEY AUTOINCREMENT,
    pid TEXT NOT NULL,
    uid INT NOT NULL,
    flag TEXT NOT NULL,
    time INT NOT NULL,
    json TEXT
);

CREATE INDEX idx_records_pid ON records(pid);
CREATE INDEX idx_records_uid ON records(uid);
CREATE INDEX idx_records_flag ON records(flag);
CREATE INDEX idx_records_time ON records(time);
