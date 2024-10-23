CREATE TABLE
  users (
    account_address CHAR(64) UNIQUE NOT NULL,
    is_legacy BOOLEAN NOT NULL
  );

CREATE TABLE balance (
    account_address CHAR(64) REFERENCES users(account_address) ON DELETE CASCADE,
    balance BIGINT NOT NULL,
    chain_timestamp TIMESTAMP NOT NULL,
    db_version BIGINT NOT NULL,
    epoch_number BIGINT NOT NULL
);
