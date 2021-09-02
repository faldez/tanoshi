use std::io;

use serde::{de::DeserializeOwned, Serialize};

pub fn write_err(message: String) {
    eprintln!("{}", message);
}

pub fn write_object<T: Serialize>(data: T) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = ron::to_string(&data)?;
    println!("{}", serialized);

    Ok(())
}

pub fn read_object<T: DeserializeOwned>() -> Result<T, Box<dyn std::error::Error>> {
    let mut serialized = String::new();
    io::stdin().read_line(&mut serialized)?;
    Ok(ron::from_str(&serialized)?)
}
