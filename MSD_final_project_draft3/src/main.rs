use csv::ReaderBuilder;
use serde::Deserialize;
use std::collections::{HashSet, HashMap};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct MSD {
    unknown: String,
    user_id: String,
    song_id: String,
    listen_count: String,
    track_id: String,
    artist_id: String,
    artist_name: String,
    title: String,
}
//function to read csv and convert it into a dataframe 
fn read_msd(file: &File) -> Result<Vec<MSD>, Box<dyn Error>> {
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

//function to find users who have listened to inputed song 
fn songs_to_users(song_title: &str, data: &[MSD]) -> HashSet<String> { //[MSD] is slice of struct (only need user_id and song title from df)
    let mut user_ids_set = HashSet::new(); //initialize hashset to store user_ids (hashset used because it makes sure each user_id is unique)

    for record in data { //iterates through each line in MSD
        if record.title == song_title { //finds titles that match inputted title
            user_ids_set.insert(record.user_id.clone()); //inserts into hashset
        }
    }
    user_ids_set //return hashset
}

//function to take user_ids_set, and find songs each user listens to
fn users_to_songs(users: &HashSet<String>, data: &[MSD]) -> HashMap<String, HashSet<String>> { //takes hashset of users from previous function
    let mut user_songs_hm: HashMap<String, HashSet<String>> = HashMap::new(); //store as hashmap (string = user id, hashset = songs)

    for record in data { //iterate through MSD
        if users.contains(&record.user_id) {  //checks if user is in hashset
            match user_songs_hm.get_mut(&record.user_id) { //checks if user already has set of songs in hashmap
                Some(song_set) => { //if user has set of songs 
                    song_set.insert(record.title.clone()); //add new song to set (.clone() because ownership is required for .insert())
                }
                None => { //if user does not already have hashset with songs
                    user_songs_hm.insert(record.user_id.clone(), { //adds user id to hashmap
                        let mut song_set = HashSet::new(); //creats hashset to store songs
                        song_set.insert(record.title.clone()); //adds song to hashset
                        song_set 
                    });
                }
            }
        }
    }
    user_songs_hm //return hashmap
}

fn most_popular_song(user_songs_hm: &HashMap<String, HashSet<String>>, exclude_input: &str) -> Option<(String, usize)> { //input is user_songs hashmap, input song, returns tuple (song name, popularity)
    let mut song_score: HashMap<String, usize> = HashMap::new(); //initialize hashset with song as key and popularity as value

     for songs in user_songs_hm.values() { //iterates through hashsets in user_songs_hm
        for song in songs { //iterate through each song in hashset
            if song != exclude_input { //excludes input song from calculations
                
                if let Some(count) = song_score.get_mut(song) { //attempt to use .get_mut to find song (key) in hashmap song_score
                    *count += 1;  //if song exists as key in hashet, add one to value (count)
                } else {
                    song_score.insert(song.clone(), 1);  //if not, add song to hashmap as key, with value (count) as 1
                }
            }
        }
    }
    let mut most_popular = None; //will become the tuple that stores the most popular song. Starts at none it is an option.
    let mut top_count = 0; //count to find most popular song

    for (song, count) in song_score {  //iterates through every song an count
        if count > top_count { //finds most popular by checking for each song and count, what count is larger
            most_popular = Some((song, count));
            top_count = count;
        }
    }
    most_popular //returns tuple of most popular song 
}

//function that reccomends songs if they do not have many users
//it takes whatever users the input song has, finds the 3 most popular songs, finds every user that listened to those 3 songs, then finds the most popular songs among them
fn find_more_songs(input_song: &str, data: &[MSD]) {
    let users = songs_to_users(input_song, data); //find users for input song

    //this code only runs if there are not enough users that have listened to the input song (>= 5)
    if users.len() >= 5 {
        println!("Song is popular, no need for deeper analysis");
        return;
    }

    let mut user_songs_hm = users_to_songs(&users, data); //find songs for each user who listened to the input song

    //call "fn most_popular" to find 3 most popular songs
    let mut top_songs: Vec<String> = vec![]; //intitialize vector to store top songs
    for _ in 0..3 { //underscore means value not needed
        if let Some((most_popular, _)) = most_popular_song(&user_songs_hm, input_song) { //underscore means ignore second value in tuple
            top_songs.push(most_popular.clone());
            for songs in user_songs_hm.values_mut() { //iterate through every song in hasmap
                songs.remove(&most_popular); //remove the most popular song so that it is not included in the next iteration
            }
        } else {
            break;
        }
    }
    //if no top songs found
    if top_songs.is_empty() {
        println!("No popular songs found");
        return;
    }

    //finds users who have listened to top songs
    let mut top_users = HashSet::new(); //create hashset to store users
    for song in &top_songs { //iterate through top songs
        let users = songs_to_users(song, data); //find all users for those songs
        for user in users { //iterate through each user
            top_users.insert(user); //add user to hashset
        }
    }
    let top_user_songs = users_to_songs(&top_users, data);

    //finds most popular songs for users 
    if let Some((most_popular, count)) = most_popular_song(&top_user_songs, "EMPTY") { //Empty because exclude input not needed
        println!("The most popular song among users who liked the top 3 songs is: '{}' with {} listeners.", most_popular, count);
    } else {
        println!("No songs found for users of the top 3 songs.");
    }
}


fn main() {
    let file = match File::open("src/merged_data.csv") {
        Ok(open_file) => open_file,
        Err(failed) => {
            eprintln!("Problem opening file");
            return; //stop code from running if error
        }
    };

    let data = match read_msd(&file) {
        Ok(create_data) => create_data,
        Err(failed) => {
            eprintln!("Problem reading MSD");
            return;
        }
    };

    let input_song = "Imagine"; //The Foundation for <=5 Imagine for >5
    let users = songs_to_users(input_song, &data);
    find_more_songs(input_song, &data);
    if users.len() > 5 { //only works if more than 5 users
        let user_songs_hm = users_to_songs(&users, &data);
        if let Some((song, count)) = most_popular_song(&user_songs_hm, input_song) {
            println!("Recommended song for '{}' is '{}' with {} listens", input_song, song, count);
        }
    } else {
        return;
    }
}

//write code to fix most_popular when there is a tie, and make sure find_more_songs to give consisten answers