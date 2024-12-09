use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MSD {
    unknown: String, //first column just contains number in list
    user_id: String, //a unique ID for each user connected to their listening habits
    song_id: String, //unique song ID specific to a song a user is listening to
    listen_count: String, //how many times the song was played by user
    track_id: String, //another unique ID for the song
    artist_id: String, //unique id for artist
    artist_name: String, //artists actual name
    title: String, //title of the song
}

//function to read csv and convert it into a dataframe 
fn read_msd(file: File) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true) .from_reader(file); //csv has column labels
    let mut data: Vec<MSD> = Vec::new(); //create new MSD df
    let mut line_count = 0; //initialize line count (limits how much of the df is read because it is so large)
    for result in rdr.records() {
        match result {
            Ok(record) => {
                match record.deserialize(None) {
                    Ok(element) => {
                        data.push(element);
                        line_count += 1; //add line to linecount
                    }
                    Err(failed) => {
                        eprintln!("Error deserializing file");
                        continue;
                    }
                }
            }
            Err(failed) => {
                eprintln!("Error reading file");
                continue;
            }
        }
        if line_count >= 100 { //stops after set number of lines read
            break; 
        }
    }

    //print df
    for element in data {
        println!("{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
                 element.unknown, element.user_id, element.song_id,
                 element.listen_count, element.track_id, element.artist_id, element.artist_name, element.title);
    }
    Ok(())
}

fn main() {
    let file = match File::open("src/merged_data.csv") { //file path
        Ok(success) => success, 
        Err(failed) => {
            eprintln!("Problem opening file");
            return; //stop if file cannot be opened
        }
    };
    if let Err(failed) = read_msd(file) {
        eprintln!("Problem reading MSD"); //can't read file
    }
}