use std::{env, str::FromStr, time::SystemTime};

use chrono::Utc;
use futures_util::stream::StreamExt;
use mongodb::{
    bson::{doc, extjson::de::Error, from_document, oid::ObjectId, DateTime},
    results::{InsertOneResult, UpdateResult},
    Client, Collection,
};

use crate::models::booking_model::{Booking, FullBooking};
use crate::models::dog_model::Dog;
use crate::models::owner_model::Owner;

pub struct Database {
    booking: Collection<Booking>,
    dog: Collection<Dog>,
    owner: Collection<Owner>,
}

// impl adalah implementasi dari struct Database, dan fungsi ini untuk inisialisasi database
impl Database {

    // fungsi untuk inisialisasi database
    pub async fn init() -> Self {
        let uri = match env::var("MONGODB_URI") {
            Ok(v) => v.to_string(),
            Err(_) => "mongodb://localhost:27017/?directConnection=true".to_string(),
        };

        // membuat client mongodb
        let client = Client::with_uri_str(&uri).await.unwrap();
        let db = client.database("dog_walking");

        let booking: Collection<Booking> = db.collection("booking");
        let dog: Collection<Dog> = db.collection("dog");
        let owner: Collection<Owner> = db.collection("owner");

        // mengembalikan struct Database
        Database {
            booking,
            dog,
            owner,
        }
    }

    // self disini mengarah ke database nya
    // kenapa menggunakan async? karena operasi database biasanya memakan waktu, jadi kita menggunakan async supaya tidak blocking thread utama
pub async fn create_owner(&self, owner: Owner) -> Result<InsertOneResult, mongodb::error::Error> {
    self.owner.insert_one(owner).await
}

pub async fn create_dog(&self, dog: Dog) -> Result<InsertOneResult, mongodb::error::Error> {
    self.dog.insert_one(dog).await
}

pub async fn create_booking(&self, booking: Booking) -> Result<InsertOneResult, mongodb::error::Error> {
    self.booking.insert_one(booking).await
}

pub async fn cancel_booking(&self, booking_id: &str) -> Result<UpdateResult, mongodb::error::Error> {
    let oid = ObjectId::from_str(booking_id)?;
    self.booking
        .update_one(doc! { "_id": oid }, doc! { "$set": { "cancelled": true } })
        .await
}

pub async fn get_bookings(&self) -> Result<Vec<FullBooking>, mongodb::error::Error> {
    let now: SystemTime = Utc::now().into();

    let mut cursor = self
        .booking
        .aggregate(
            vec![
                doc! {
                    "$match": {
                        "cancelled": false,
                        "start_time": { "$gte": DateTime::from_system_time(now) }
                    }
                },
                doc! {
                    "$lookup": {
                        "from": "owner",
                        "localField": "owner",
                        "foreignField": "_id",
                        "as": "owner"
                    }
                },
                doc! { "$unwind": "$owner" },
                doc! {
                    "$lookup": {
                        "from": "dog",
                        "localField": "owner._id",
                        "foreignField": "owner",
                        "as": "dogs"
                    }
                },
            ],
        )
        .await?;

    let mut bookings = Vec::new();

    while let Some(result) = cursor.next().await {
        let doc = result?;
        let booking: FullBooking = from_document(doc)?;
        bookings.push(booking);
    }

    Ok(bookings)
}
}