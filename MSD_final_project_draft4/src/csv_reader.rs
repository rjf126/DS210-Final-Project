use csv::ReaderBuilder;
use serde::Deserialize;
use std::collections::{HashSet, HashMap};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
pub struct MSD {
    pub unknown: String, //first column just contains row number 
    pub user_id: String, //a unique ID for each user connected to their listening habits
    pub song_id: String, //unique song ID specific to a song a user is listening to
    pub listen_count: String, //how many times the song was played by user
    pub track_id: String, //another unique ID for the song
    pub artist_id: String, //unique id for artist
    pub artist_name: String, //artists actual name
    pub title: String, //title of the song
}
//function to read csv and convert it into a dataframe 
pub fn read_msd(file: &File) -> Result<Vec<MSD>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file); //csv has column labels
    let mut data: Vec<MSD> = Vec::new(); //create new MSD df
    let mut line_count = 0; //initialize line count (limits how much of the df is read because it is so large)

    for result in rdr.records() {
        match result {
            Ok(record) => match record.deserialize(None) {
                Ok(element) => {
                    data.push(element);
                    line_count += 1; //add line to linecount
                }
                Err(failed) => {
                    eprintln!("Error deserializing record");
                    continue;
                }
            },
            Err(failed) => {
                eprintln!("Error reading record");
                continue;
            }
        }
        if line_count >= 200000 { //stops after set number of lines read
            break;
        }
    }

    Ok(data) //return data for use in future functions
}