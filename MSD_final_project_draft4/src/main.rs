mod csv_reader;

use csv_reader::{read_msd, MSD};
use std::collections::{HashSet, HashMap};
use std::error::Error;
use std::fs::File;

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

fn most_popular_song(user_songs_hm: &HashMap<String, HashSet<String>>, exclude_input: &str) -> Option<(String, usize)> {
    let mut song_score: HashMap<String, usize> = HashMap::new(); //initialize hashset with song as key and popularity as value

    for songs in user_songs_hm.values() { //iterates through hashsets in user_songs_hm
        for song in songs { //iterate through each song in hashset
            if song != exclude_input { //excludes input song from calculations
                if let Some(count) = song_score.get_mut(song) { //attempt to use .get_mut to find song (key) in hashmap song_score
                    *count += 1;  //if song exists as key in hashmap, add one to value (count)
                } else {
                    song_score.insert(song.clone(), 1);  //if not, add song to hashmap as key, with value (count) as 1
                }
            }
        }
    }

    let mut most_popular = None; //will become the tuple that stores the most popular song. Starts at none, it is an option.
    let mut top_count = 0; //count to find most popular song

    for (song, count) in song_score {  //iterates through every song and count
        if count > top_count
            || (count == top_count && match &most_popular {
                Some((song_name, _)) => song < *song_name, //if song is lexicographically smaller, current song is more popular
                None => true,
            })
        {
            most_popular = Some((song, count)); //update most_popular if new song is more popular or has a lexicographically smaller name
            top_count = count; //update the top count
        }
    }

    most_popular //returns tuple of most popular song
}

//function that reccomends songs if they do not have many users
//it takes whatever users the input song has, finds the 3 most popular songs, finds every user that listened to those 3 songs, then finds the most popular songs among them
fn find_more_songs(input_song: &str, data: &[MSD]) -> Option<(String, usize)> {
    let users = songs_to_users(input_song, data); //find users for input song

    //this code only runs if there are not enough users that have listened to the input song (>= 5)
    if users.len() >= 5 {
        println!("Song is popular, no need for deeper analysis");
        return None;
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
        return None;
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
    if let Some((most_popular, count)) = most_popular_song(&top_user_songs, "EMPTY") { //input empty because exclude_input not needed
        return Some((most_popular, count)); //return most popular song as tuple
    } else {
        return None;
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

    //printing fn most_popular (only works if more than 5 users)
    if users.len() > 5 { 
        let user_songs_hm = users_to_songs(&users, &data);
        if let Some((song, count)) = most_popular_song(&user_songs_hm, input_song) {
            println!("Recommended song for '{}' is '{}' with {} listens", input_song, song, count);
        }
    } else {
        println!("Song is not popular, looking for better recommendations...");
    }

    //prints fn find_more_songs (<5 users)
    match find_more_songs(input_song, &data) {
        Some((song, count)) => {
            println!("Most popular recommended song is {} with {} listeners", song, count);
        },
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //all tests passed

    fn fake_data() -> Vec<MSD> {
        vec![
            //0's represent unimportant data
            MSD { 
                //user1 listens to Song A
                unknown: "0".to_string(),
                user_id: "user1".to_string(),
                song_id: "0".to_string(),
                listen_count: "0".to_string(),
                track_id: "0".to_string(),
                artist_id: "0".to_string(),
                artist_name: "0".to_string(),
                title: "Song A".to_string(),
            },
            MSD { 
                //user2 listens to Song A
                unknown: "0".to_string(),
                user_id: "user2".to_string(),
                song_id: "0".to_string(),
                listen_count: "0".to_string(),
                track_id: "0".to_string(),
                artist_id: "0".to_string(),
                artist_name: "0".to_string(),
                title: "Song A".to_string(),
            },
            MSD { 
                //user1 listens to Song B
                unknown: "0".to_string(),
                user_id: "user1".to_string(),
                song_id: "0".to_string(),
                listen_count: "0".to_string(),
                track_id: "0".to_string(),
                artist_id: "0".to_string(),
                artist_name: "0".to_string(),
                title: "Song B".to_string(),
            },
            MSD {
                //user3 listens to Song C
                unknown: "0".to_string(),
                user_id: "user3".to_string(),
                song_id: "0".to_string(),
                listen_count: "0".to_string(),
                track_id: "0".to_string(),
                artist_id: "0".to_string(),
                artist_name: "0".to_string(),
                title: "Song C".to_string(),
            },
        ]
    }
    
    #[test]
    fn test_songs_to_users() {
        let data = fake_data();
        let users = songs_to_users("Song A", &data);
        //two users listen to Song A (user1, user2)
        assert_eq!(users.len(), 2);
        //for song A, user1 and user2 listened (checks if they exist in users)
        assert!(users.contains("user1"));
        assert!(users.contains("user2"));
    }

    #[test]
    fn test_users_to_songs() {
        let data = fake_data();
        //checks user1 and user2
        //.intro_iter().collect() turns data it into hashset
        let users: HashSet<String> = ["user1".to_string(), "user2".to_string()].into_iter().collect();
        let user_songs = users_to_songs(&users, &data);
    
        //user 1 listened to Song A and Song B
        let expected_user1: HashSet<String> = ["Song A".to_string(), "Song B".to_string()].into_iter().collect();

        //user 2 listened to Song A
        let expected_user2: HashSet<String> = ["Song A".to_string()].into_iter().collect();
    
        //user_songs.get("user1") should be Song A, Song B
        assert_eq!(user_songs.get("user1"), Some(&expected_user1));
        //user_songs.get("user2") should just be song A
        assert_eq!(user_songs.get("user2"), Some(&expected_user2));
        
        //makes sure there are only 2 users
        assert_eq!(user_songs.len(), 2);
    }

    #[test]
    fn test_most_popular_song() {
        let data = fake_data();
        let users: HashSet<String> = ["user1".to_string(), "user2".to_string()].into_iter().collect();
        let user_songs_hm = users_to_songs(&users, &data);
        let most_popular = most_popular_song(&user_songs_hm, "Song A");
        //two people listen to Song A, most popular outside of that is Song B with 1 play
        assert_eq!(most_popular, Some(("Song B".to_string(), 1)));
    }
    #[test]
    fn test_find_more_songs() {
        let data = fake_data();
        let actual = find_more_songs("Song B", &data);
        assert_eq!(actual, Some(("Song A".to_string(), 2))); //Song A should be the most popular with 2 users
    }

}