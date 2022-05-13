#!/bin/bash

source .env.test

function backend {
cat << EOF > backend/.env
MONGO_DB_URL=$TEST_MONGO_DB_URL
MONGO_DATABASE_NAME=$TEST_MONGO_INITDB_DATABASE

SERVER_PORT=$TEST_SERVER_PORT
SERVER_HOST=$TEST_SERVER_HOST
EOF
}

backend