#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE ROLE khata_ro LOGIN PASSWORD '${KHAATA_RO_DB_PASSWORD}';
EOSQL
