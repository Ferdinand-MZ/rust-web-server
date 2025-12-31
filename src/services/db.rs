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
    pub async fn create_owner(&self, owner: Owner) -> Result<InsertOneResult, Error> {
        let result = self
            .owner
            .insert_one(owner)
            .await
            .ok()
            .expect("Failed to insert owner");

        Ok(result)
    }


    pub async fn create_dog(&self, dog: Dog) -> Result<InsertOneResult, Error> {
        // memasukkan data dog ke dalam collection dog
        let result = self
            .dog
            .insert_one(dog)
            .await
            .ok()
            .expect("Failed to insert dog");

        Ok(result)
    }


    pub async fn create_booking(&self, booking: Booking) -> Result<InsertOneResult, Error> {
        // memasukkan data booking ke dalam collection booking
        let result = self
            .booking
            .insert_one(booking)
            .await
            .ok()
            .expect("Failed to insert booking");

        Ok(result)
    }


    pub async fn cancel_booking(&self, booking_id: &str) -> Result<UpdateResult, Error> {
        // memperbarui field cancelled menjadi true berdasarkan booking_id
        let result = self
            .booking
            .update_one(doc! {
                "_id" : ObjectId::from_str(booking_id).expect("Failed to parse booking_id")
            }, doc! {
                "$set": { "cancelled": true }
            })
            .await
            .ok()
            .expect("Failed to cancel booking");

        Ok(result)
    }

    // fungsi untuk mendapatkan semua booking yang belum dibatalkan dan start_time nya di masa depan
    pub async fn get_bookings(&self) -> Result<Vec<FullBooking>, Error> { //ini memakai vector karena kita mengembalikan banyak data booking
        let now: SystemTime = Utc::now().into();

        // melakukan agregasi untuk mendapatkan data booking lengkap dengan data owner dan dog
        let mut results = self
            .booking
            .aggregate(
    vec![
                doc! {
                    "$match": {
                        "cancelled" : false,
                        "start_time": { 
                            "$gte": DateTime::from_system_time(now)
                        }
                    }
                },
                doc! {
                    "$lookup": doc!{
                        "from": "owner",
                        "localField": "owner",
                        "foreignField": "_id",
                        "as": "owner"
                    }
                },
                doc! {
                    "$unwind": doc! {
                        "path": "$owner"
                    }
                },
                doc!{
                    "$lookup" : doc!{
                        "from": "dog",
                        "localField": "owner._id",
                        "foreignField": "owner",
                        "as": "dogs"
                    }
                },
                ],
                //None
            )
            .await
            .ok()
            .expect("Failed getting full bookings");

        let mut bookings: Vec<FullBooking> = Vec::new(); // Membuat vector kosong untuk menyimpan data booking lengkap

        // apa yang terjadi disini adalah kita melakukan iterasi terhadap hasil agregasi dari query di atas atau bahasa normalnya kita mengambil setiap dokumen hasil query satu per satu
        // untuk setiap dokumen hasil query, kita mengonversinya menjadi struct FullBooking dan menambahkannya ke dalam vector bookings
        while let Some(result) = results.next().await {
            match result {
                Ok(doc) => {
                    let booking: FullBooking = from_document(doc).expect("Failed Converting document to full booking");
                    bookings.push(booking);
                }

                Err(e) => panic!("Error retrieving booking: {}", e), ////panic merupakan macro untuk menghentikan program dan menampilkan pesan error 
            }
        }

        Ok(bookings)
    }
}