#!/bin/bash

# Создание пользователя для БД
mongo -- "$MONGO_INITDB_DATABASE" <<EOF
    db.createUser({
        user: "$MONGO_USER",
        pwd: "$MONGO_USER_PASSWORD",
        roles: [
            {
                role: "$MONGO_USER_ROLE",
                db: "$MONGO_INITDB_DATABASE",
            },
        ],
    });
EOF

# Создание коллекций и установка индексов
mongo --username $MONGO_USER --password $MONGO_USER_PASSWORD --authenticationDatabase $MONGO_INITDB_DATABASE $MONGO_INITDB_DATABASE <<EOF
    db.createCollection("users");
    db.users.createIndex(
        {
            "username": 1
        }, 
        {
            "unique": true, 
            "partialFilterExpression": {
                "username": {
                    \$type: "string"
                }
            }
        }
    );
    db.users.insertOne(
        {
            "_id": "82e1f645-9b38-4773-8fe0-6d98c756f920",
            "username": "shavedkiwi",
            "hash": "\$argon2i\$v=19\$m=4096,t=3,p=1\$polzlXI0YXGFxBp2aFq8orG8XG/VhwlBTlyLP+ZSrCE\$$MONGO_JOKERHUB_SITH_HASH",
            "level": "sith",
            "tariff": "enterprice",
            "created_at": "2022-05-21T08:49:20.516119282",
            "updated_at": "2022-05-21T08:49:20.516119963"
        }
    );

    db.createCollection("anecdote");
    db.anecdote.createIndex(
        {
            "text": 1
        }, 
        {
            "unique": true, 
            "partialFilterExpression": {
                "text": {
                    \$type: "string"
                }
            }
        }
    );

    db.createCollection("punch");
    db.punch.createIndex(
        {
            "setup": 1,
            "punchline": 1,
        }, 
        {
            "unique": true, 
            "partialFilterExpression": {
                "setup": {
                    \$type: "string"
                },
                
                "punchline": {
                    \$type: "string"
                },
            }
        }
    );
EOF