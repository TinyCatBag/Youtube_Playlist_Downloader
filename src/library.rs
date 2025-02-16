use id3::{Tag, TagLike};
use reqwest::{Client, get};
use std::{collections::HashMap, env::current_dir, fs::{create_dir, read_dir, read_to_string}, process::Command, time::Duration};
use serde_json::{from_str, Value};
use indicatif::ProgressBar;

#[derive(Debug)]
pub struct Playlist {
    title: String,
    videos: Vec<(String, String)>, //Video title and ID
    total_videos: u64,
}


async fn get_playlist_value(playlist_id: &str, continuation_token: Option<&str>, ) -> Value{
    let pattern = "var ytInitialData = ";

    match continuation_token {
        None =>{
            let response = get(format!("https://www.youtube.com/playlist?list={playlist_id}"))
                .await.unwrap().text().await.unwrap();
            
            let start_json = response.find(pattern).expect("Didn't find start of json");
            let second = &response[start_json..];
            let end_json = second.find(";").expect("Didn't end find of json");
            let result = &second[pattern.len()..end_json];
            from_str(&result).unwrap()
        }
        Some(continuation_token) => {
            let response = Client::new().post("https://www.youtube.com/youtubei/v1/browse?prettyPrint=false")
                .header("Content-Type", "application/json")
                .body(format!
                    (r#"{{"context":{{"client":{{"clientName":"WEB","clientVersion":"2.20241205.01.00"}}}},
                    "continuation":"{continuation_token}"}}"#))
                .send().await.unwrap().text().await.unwrap();
                from_str(&response[..]).unwrap()
        }
    }
}

fn get_playlist_title(value: &Value) -> String{
    value["metadata"]["playlistMetadataRenderer"]["title"]
    .to_string().replace("\"", "")
}

fn get_continuation_token(value: &Value, first: bool) -> String {
    match first {
        true => {
            value
                ["continuationItemRenderer"]["continuationEndpoint"]                // getting into the continuation token
                ["continuationCommand"]["token"]                              // At last here is the token
                .to_string().replace("\"", "")                            // Formating
        }
        false => {
            value
                ["continuationItemRenderer"]["continuationEndpoint"]        // Unpacking the token
                ["continuationCommand"]["token"]                      // Here lies the token
                .to_string().replace("\"", "")                    // Formatting
        }
    }
}

fn get_video(value: &Value, /* id: &usize , */ first: bool) -> (String, String){
    match first {
        true => {
            let bloat = &value
                ["playlistVideoRenderer"];

            (
            bloat
                ["title"]["runs"][0]                       // Useless Array
                ["text"].to_string().replace("\"", ""),
            bloat
                ["videoId"]                       // Useless Array
                .to_string().replace("\"", ""),
            )                    // Formatting
        },
        false => {
            let bloat = &value
                ["playlistVideoRenderer"];
            (
            bloat
                ["title"]["runs"][0]                                        // Video title
                ["text"].to_string().replace("\"", ""),           // Formatting
            
            bloat
                ["videoId"]                                           // Video title
                .to_string().replace("\"", ""),                   // Formatting
            )
        }
    }
}

fn get_playlist_array(value: &Value, first: bool) -> &Vec<Value>{
    match first {
    true => {
        value
            ["contents"]["twoColumnBrowseResultsRenderer"]["tabs"][0]           // Useless Array
            ["tabRenderer"]["content"]["sectionListRenderer"]["contents"][0]    // Weird continuation token
            ["itemSectionRenderer"]["contents"][0]                              // Useless Array
            ["playlistVideoListRenderer"]["contents"]                     // Video index in playlist (Last index is the continuation token)
            .as_array().expect("Failed to find video array")               // To Array and unwrap
        }
    false => {
        value
            ["onResponseReceivedActions"][0]                                    // Useless Array
            ["appendContinuationItemsAction"]["continuationItems"]        // Video Array
            .as_array().expect("Failed to find video array")               // To Array and unwrap
        }
    }
}

fn get_total_videos(value: &Value) -> u64 {
    from_str::<u64>(value["header"]["pageHeaderRenderer"]["content"]
        ["pageHeaderViewModel"]["metadata"]
        ["contentMetadataViewModel"]["metadataRows"][1]
        ["metadataParts"][1]["text"]["content"]
        .to_string().replace("\"", "")
        .split_whitespace().clone().collect::<Vec<_>>()[0]).unwrap()
}

async fn get_all_videos(playlist_id: &str, continuation_token: Option<&str>) -> Playlist{
    let mut playlist = Playlist{
        title: String::new(),
        videos: Vec::new(),
        total_videos: 0
    };
    format!("Getting all videos from: {}", playlist_id);

    match continuation_token {
        None => {
            let first = get_playlist_value(playlist_id, None).await;
            playlist.title = get_playlist_title(&first).clone();
            playlist.total_videos = get_total_videos(&first);

            for x in get_playlist_array(&first, true){
                let video = get_video(x,true);
                if video == ("null".to_string(),"null".to_string()) {
                    let token = get_continuation_token(&x, true);
                    let temp = Box::pin(get_all_videos(playlist_id, Some(&token))).await;
                    playlist.videos.extend(temp.videos.clone());
                    continue;
                }
                playlist.videos.push(video);
            }
        }
        Some(continuation_token) => {
            let first = get_playlist_value(playlist_id, Some(continuation_token)).await;

            for x in get_playlist_array(&first, false){
                let video = get_video(x,false);
                if video == ("null".to_string(),"null".to_string()) {
                    let token = get_continuation_token(&x, true);
                    playlist.videos.extend(Box::pin(get_all_videos(playlist_id, Some(&token))).await.videos.clone());
                    continue;
                }
                playlist.videos.push(video);
            }
        }
    }
    playlist
}

async fn check_playlist(id: &str) -> Playlist{
    let playlist = get_all_videos(id, None).await;
    let mut hashmap_videos: HashMap<String, &String> = HashMap::new();
    for x in &playlist.videos {
        hashmap_videos.insert(
            format!("{} - {}.mp3", playlist.title, x.0
            .replace("|", "｜")
            .replace(":", "：") 
            .replace("\\", "＂")
            .replace("/", "⧸")
            .replace("?", "？")
            ), &x.1);
    }

    let cur_dir = current_dir().unwrap();
    let dir = read_dir(&cur_dir).expect("Failed to read/find dir");
    
    let mut missing_videos: Vec<(String, String)> = Vec::new();
    let mut downloaded_videos:  HashMap<String, String> = HashMap::new();
    //let mut removed_videos: Vec<(String, String)> = Vec::new();
    let _ = create_dir(format!("{}/{}", &cur_dir.display(), playlist.title));
    for x in dir {
        let x = &x.unwrap();
        if x.path().is_file(){
            continue
        }
        for z in read_dir(x.path()).unwrap(){
            let z = z.unwrap();
            let file_name = z.file_name().into_string().unwrap();
            let file_path = z.path().to_string_lossy().into_owned();
            downloaded_videos.insert(file_name, file_path);
            //println!("{:#?}", z.file_name().into_string().unwrap());
        }
    }
    
    for x in &playlist.videos {
        let formatted_title = &format!("{} - {}.mp3", playlist.title, x.0
            .replace("|", "｜")
            .replace(":", "：") 
            .replace("\\", "＂")
            .replace("/", "⧸")
            .replace("?", "？")
            );
        if downloaded_videos.contains_key(formatted_title) {
            continue;
        }
        missing_videos.push((formatted_title.to_string(), x.1.clone()));
    }

    //for x in &downloaded_videos {
    //    let formatted_title = &format!("{} - {}.mp3", playlist.title, x.0
    //        .replace("|", "｜")
    //        .replace(":", "：") 
    //        .replace("\\", "＂")
    //        .replace("/", "⧸")
    //        .replace("?", "？")
    //        );
    //    TODO: downloaded_videos has videos from ALL playlists and we are cheching only against
    //    one so we are kinda fucked lol
    //
    //    TODO: So rn the code succesfully checks if a vid is in the playlist so yeah
    //    but still fucks around and pushes all downloaded_videos
    //    if !x.1.contains(&format!("{}/{}", cur_dir.to_string_lossy(), playlist.title)) {
    //        println!("Playlist: {}", &format!("{}/{}", cur_dir.to_string_lossy(), playlist.title));
    //        continue;
    //    }
    //    if hashmap_videos.contains_key(x.0) {
    //        println!("How?: {}", x.0);
    //        continue;
    //    }
    //    removed_videos.push((x.0.clone(), x.1.to_string()));
    //}
    println!("==========================================");
    println!("Playlist: {}", playlist.title);
    if missing_videos.len() > 0 {
        for x in &missing_videos {
        println!("Videos to Download: {:#?}", x.0);
        }
    }
    else {
        println!("No videos to download")
    }
    //println!("Videos to remove: {:#?}", downloaded_videos);
    Playlist{
        title: playlist.title,
        videos: missing_videos,
        total_videos: playlist.total_videos
    }
}

pub async fn download_playlist(id: &str) {
    let missing_videos = check_playlist(id).await;
    
    for videos in missing_videos.videos {
        let mut downloader = Command::new("yt-dlp")
            .args(["--embed-thumbnail", 
                "-x" ,"--audio-format", "mp3",
                "-o", &format!("{0}/{0} - %(title)s.%(ext)s", missing_videos.title,)[..],
                &format!("https://www.youtube.com/watch?v={}", videos.1)])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn().unwrap();
        let bar = ProgressBar::new_spinner().with_message(format!("Downloading: {:#?}", videos.0));
            bar.enable_steady_tick(Duration::from_millis(100));
        
        downloader.wait().expect("Downloader failed =3");
            bar.set_message(format!("Finished downloading: {:#?}", videos.0));
            bar.finish();
        //let formatted_title = &format!("{}", missing_videos.title, videos.0
        //    .replace("|", "｜")
        //    .replace(":", "：") 
        //    .replace("\\", "＂")
        //    .replace("/", "⧸")
        //    .replace("?", "？")
        //    );

        
        let mut tag = Tag::new();
        tag.set_artist(videos.1);
        println!("{}", &format!("{}/{}/{}", current_dir().unwrap().to_string_lossy(), missing_videos.title, videos.0));
        tag.write_to_path(
            std::path::Path::new(
                &format!("{}/{}/{}", current_dir().unwrap().to_string_lossy(), missing_videos.title, videos.0))
            , id3::Version::Id3v24).unwrap()
    }
}

pub fn read_file (path: String) -> Vec<String>{
        let binding2 = read_to_string(path).unwrap();
        let binding = binding2.lines();
        let mut playlist_ids: Vec<String> = Vec::new();
        for x in binding {
            playlist_ids.push(x.to_owned());
        }
        return playlist_ids
}
