use bson::Document;
use mongodb::bson::DateTime as MongoDateTime;
use mongodb::results::{DeleteResult, InsertOneResult, UpdateResult};
use mongodb::sync::Client;
use mongodb::{bson::doc, sync::Collection};

use crate::model::account::{security::Session, User};
use crate::{
    db::mongo::{varys::Varys, Crud},
    err_not_found, err_unauthorized,
    errors::HubError,
};

impl<'a> Crud<'a, User> for User {}

impl<'a> User {
    pub fn get_by_username(client: &Client, username: String) -> Result<User, HubError> {
        let collection: Collection<User> = Varys::get(client, Varys::Users);
        match collection.find_one(doc! { "username":  username}, None)? {
            Some(value) => Ok(value),
            None => Err(err_not_found!("user")),
        }
    }

    pub fn del_by_username(
        collection: Collection<User>,
        username: String,
    ) -> Result<DeleteResult, HubError> {
        let res = collection.delete_one(doc! { "username":  username}, None)?;

        Ok(res)
    }

    pub fn update_password(
        client: &Client,
        username: String,
        new_password_hash: String,
    ) -> Result<UpdateResult, HubError> {
        let collection: Collection<User> = Varys::get(client, Varys::Users);

        let filter = doc! {"username": username};
        let update = doc! {"$set": {"hash": new_password_hash, "updated_at": MongoDateTime::now()}};

        let res = collection.update_one(filter, update, None)?;

        Ok(res)
    }

    pub fn privilege_set(
        collection: Collection<User>,
        username: &str,
        level: &str,
    ) -> Result<UpdateResult, HubError> {
        let filter = doc! {"username": username};
        let update = doc! {"$set": {"level": level, "updated_at": MongoDateTime::now()}};

        let res = collection.update_one(filter, update, None)?;

        Ok(res)
    }
}

impl Session {
    pub fn set<'f>(&self, client: &Client) -> Result<InsertOneResult, HubError> {
        let collection: Collection<Document> = Varys::get(client, Varys::Sessions);
        let doc = bson::to_document(&self)?;
        let rersult = collection.insert_one(doc, None)?;

        Ok(rersult)
    }

    pub fn check<'f>(token: &'f str, client: &'f Client) -> Result<Session, HubError> {
        let collection: Collection<Session> = Varys::get(client, Varys::Sessions);
        match collection.find_one(doc! { "token":  token}, None)? {
            Some(value) => Ok(value),
            None => Err(err_unauthorized!("Session is not found")),
        }
    }

    pub fn roll<'f>(username: &'f str, client: &'f Client) -> Result<Vec<Session>, HubError> {
        let collection: Collection<Session> = Varys::get(client, Varys::Sessions);

        let mut cursor = collection.find(doc! {"username": username}, None)?;
        let mut result: Vec<Session> = Vec::new();

        while let Some(doc) = cursor.next() {
            result.push(doc?);
        }

        Ok(result)
    }

    pub fn drop<'f>(token: &str, client: &Client) -> Result<DeleteResult, HubError> {
        let collection: Collection<Session> = Varys::get(client, Varys::Sessions);
        match collection.delete_one(doc! { "token":  token}, None) {
            Ok(dr) => Ok(dr),
            Err(err) => Err(err_unauthorized!("Falid to drop token", err)),
        }
    }
}
