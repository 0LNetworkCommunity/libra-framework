
# Migration file naming

from https://docs.rs/sqlx/latest/sqlx/migrate/trait.MigrationSource.html:
"
In the default implementation, a MigrationSource is a directory which contains the migration SQL scripts. All these scripts must be stored in files with names using the format <VERSION>_<DESCRIPTION>.sql, where <VERSION> is a string that can be parsed into i64 and its value is greater than zero, and <DESCRIPTION> is a string.

Files that don’t match this format are silently ignored.
"
