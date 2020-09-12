CREATE TABLE items (
    id INTEGER PRIMARY KEY NOT NULL,
    subscription_id INTEGER NOT NULL,
    url VARCHAR(4096) NOT NULL,
    title VARCHAR(256) NOT NULL,
    author VARCHAR(256), -- NULLABLE
    published TIMESTAMP NOT NULL,
    updated TIMESTAMP NOT NULL,
    content VARCHAR NOT NULL,

    is_read BOOLEAN NOT NULL,
    is_starred BOOLEAN NOT NULL,

    FOREIGN KEY(subscription_id) REFERENCES subscriptions(id)
);
