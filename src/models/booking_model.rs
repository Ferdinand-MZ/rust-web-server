use mongodb::bson::{oid::ObjectId, DateTime}; //Mengimpor ObjectId dan DateTime dari modul bson di dalam crate mongodb
use serde::{Deserialize, Serialize}; //Deserialize dan Serialize digunakan untuk mengubah data menjadi format yang dapat disimpan atau ditransmisikan, seperti JSON.

use chrono::Utc; //Mengimpor Utc dari crate chrono untuk menangani zona waktu UTC
// crate adalah paket atau library di Rust, dan chrono adalah crate yang digunakan untuk manipulasi tanggal dan waktu.

use std::time::SystemTime; //Mengimpor SystemTime dari modul time di dalam crate std untuk menangani waktu sistem

use crate::models::{owner_model::Owner, dog_model::Dog}; //Mengimpor struct Owner dan Dog dari modul models di dalam crate saat ini

#[derive(Debug, Serialize, Deserialize)] //Derive digunakan untuk mengimplementasikan trait tertentu secara otomatis, dalam hal ini Debug, Serialize, dan Deserialize
pub struct Booking { // pub adalah keyword untuk membuat struct menjadi public dan diakses modul lain
    pub _id : ObjectId,
    pub owner: ObjectId,
    pub start_time: DateTime,
    pub duration_in_minutes: u8,
    pub cancelled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookingRequest { // disini kita membuat struct BookingRequest untuk menerima data dari client
    pub owner: String,
    pub start_time: String,
    pub duration_in_minutes: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullBooking { // disini kita membuat struct FullBooking untuk mengembalikan data booking lengkap ke client
    pub _id : ObjectId,
    pub owner: Owner, // disini owner bertipe Owner karena kita akan mengembalikan data owner lengkap
    pub dogs: Vec<Dog>, // disini dogs bertipe vector of Dog karena satu owner bisa memiliki banyak dog
    pub start_time: DateTime,
    pub duration_in_minutes: u8,
    pub cancelled: bool,
}

impl TryFrom<BookingRequest> for Booking { //Mengimplementasikan trait TryFrom untuk mengonversi BookingRequest menjadi Booking
    type Error = Box<dyn std::error::Error>; //Menentukan tipe error yang dapat terjadi selama konversi

    fn try_from(item: BookingRequest) -> Result<Self, Self::Error> { //Fungsi untuk melakukan konversi
        let chrono_datetime: SystemTime = chrono::DateTime::parse_from_rfc3339(&item.start_time) // Mengonversi string start_time menjadi DateTime menggunakan format RFC 3339
        .map_err(|err|format!("Failed to parse start_time: {}", err))? //Mengonversi string start_time menjadi DateTime
        .with_timezone(&Utc) // Mengatur zona waktu ke UTC
        .into(); // Mengonversi ke SystemTime
    
        Ok(Self {
            _id: ObjectId::new(), // Membuat ObjectId baru untuk field _id
            owner: ObjectId::parse_str(&item.owner).expect("Failed to parse owner"), // Mengonversi string owner menjadi ObjectId
            start_time: DateTime::from(chrono_datetime), // Mengonversi SystemTime menjadi DateTime
            duration_in_minutes: item.duration_in_minutes,
            cancelled: false, // Mengatur field cancelled ke false secara default
            
        })
    }
}