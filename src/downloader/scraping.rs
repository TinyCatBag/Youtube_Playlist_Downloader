use std::{path::{Path, PathBuf}, sync::Arc};
use log::{debug, warn, trace, info};
use reqwest::{Client, get};
use std::{env::current_dir, fs::create_dir, io::ErrorKind, vec};
use serde_json::{from_str, Value};

#[derive(Clone)]
pub struct Playlist {
    pub videos: Vec<Arc<Video>>,
    pub title: String,
    pub path: PathBuf,
    pub id: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Video {
    pub title: String,
    pub formatted_title: String,
    pub author: String,
    pub id: String,
    pub path: PathBuf,
}

impl Playlist {
    pub async fn new(id: impl Into<String>) -> Self{
        let id: String = id.into();
        //TODO: Find a smart way to not request the entire value JUST for the title :3
        let value = Self::get_value(&id, None).await;
        trace!("Playlist value: {value}");
        let title = Self::get_title(&value);
        let mut path = current_dir().unwrap();
        path.push(&title);
        debug!("Playlist title: {title}, path: path");

        match create_dir(&path) {
            Ok(_) => {  info!("Succesfully created playlist directory: {}", &path.display()) },
            Err(err) => {
                if err.kind() == ErrorKind::AlreadyExists {
                    warn!("Playlist directory alread exists: {}", &path.display())
                }
                else {
                    panic!("Error with creating playlist directory: {}", err);
                }
            },
        }
        
        let videos = Self::get_videos(&id, None, &title, &path).await;
        Self {
            videos,
            title,
            path,
            id,
        }
    }
    fn get_title(value: &Value) -> String{
        value["metadata"]["playlistMetadataRenderer"]["title"]
        .to_string().replace("\"", "")
    }
    async fn get_value(playlist_id: &str, continuation_token: Option<&str>, ) -> Value{
        let pattern = "var ytInitialData = ";
        let value = match continuation_token {
            None =>{
                debug!("Fetching playlist value without a cont token");
                let response = get(format!("https://www.youtube.com/playlist?list={playlist_id}"))
                    .await.unwrap().text().await.unwrap();
                debug!("Finding json");
                let start_json_index = response.find(pattern).expect("Didn't find start of json");
                let start_json = &response[start_json_index..];
                let end_json = start_json.find(";").expect("Didn't find end of json");
                let full_json = &start_json[pattern.len()..end_json];
                debug!("start_json_index: {start_json_index}, end_json: {end_json}");
                trace!("Full json: {full_json}");
                debug!("Making value from string");
                from_str(&full_json).unwrap()
            }
            Some(continuation_token) => {
                debug!("Fetching playlsit value with a continuation token");
                debug!("Continuation token: {continuation_token}");
                let response = Client::new().post("https://www.youtube.com/youtubei/v1/browse?prettyPrint=false")
                    .header("Content-Type", "application/json")
                    .body(format!
                        (r#"{{"context":{{"client":{{"clientName":"WEB","clientVersion":"2.20250523.01.00"}}}},
                        "continuation":"{continuation_token}"}}"#))
                    .send().await.unwrap().text().await.unwrap();
                trace!("Response: {response}");
                debug!("Parsing response to Value");
                    from_str(&response[..]).unwrap()
            }
        };
        value
    }
    fn get_continuation_token(value: &Value) -> String {
        // contents.twoColumnBrowseResultsRenderer.tabs.0.tabRenderer.content.sectionListRenderer.contents.0.itemSectionRenderer.contents.0.playlistVideoListRenderer.contents.100.
        // continuationItemRenderer.continuationEndpoint.commandExecutorCommand.commands.1.continuationCommand
        // ^^ Path to token ^^
        
        // trace!("Token Path Trace:{:#?}", value["continuationItemRenderer"]);
        // trace!("Token Path Trace:{:#?}", value["continuationItemRenderer"]["continuationEndpoint"]);
        // trace!("Token Path Trace:{:#?}", value["continuationItemRenderer"]["continuationItemRenderer"]["continuationEndpoint"]["commandExecutorCommand"]);
        // trace!("Token Path Trace:{:#?}", value["continuationItemRenderer"]["continuationItemRenderer"]["continuationEndpoint"]["commandExecutorCommand"]["commands"]);
        // trace!("Token Path Trace:{:#?}", value["continuationItemRenderer"]["continuationItemRenderer"]["continuationEndpoint"]["commandExecutorCommand"]["commands"][1]);
        // trace!("Token Path Trace:{:#?}", value["continuationItemRenderer"]["continuationItemRenderer"]["continuationEndpoint"]["commandExecutorCommand"]["commands"][1]["continuationCommand"]);
        // trace!("Token Path Trace:{:#?}", value["continuationItemRenderer"]["continuationItemRenderer"]["continuationEndpoint"]["commandExecutorCommand"]["commands"][1]["continuationCommand"]["token"]);
        debug!("Getting continuation token");
        value
            ["continuationItemRenderer"]["continuationEndpoint"]["commandExecutorCommand"]  // Some bloat
            ["commands"][1]["continuationCommand"]["token"]                                 // Token
            .to_string().replace("\"", "")                                                  // Formating

    }
    fn get_array(value: &Value, first: bool) -> &Vec<Value>{
        match first {
            // contents.twoColumnBrowseResultsRenderer.tabs.0.tabRenderer.content.sectionListRenderer.contents.0.itemSectionRenderer.contents.0.playlistVideoListRenderer.contents.
            // onResponseReceivedActions.0.appendContinuationItemsAction.continuationItems.
            // ^^ Path to video array ^^
        true => {
            // trace!("Array Path Trace: {:#?}", value["contents"]);
            // trace!("Array Path Trace: {:#?}", value["contents"]["twoColumnBrowseResultsRenderer"]);
            // trace!("Array Path Trace: {:#?}", value["contents"]["twoColumnBrowseResultsRenderer"]["tabs"]);
            // trace!("Array Path Trace: {:#?}", value["contents"]["twoColumnBrowseResultsRenderer"]["tabs"][0]["tabRenderer"]);
            // trace!("Array Path Trace: {:#?}", value["contents"]["twoColumnBrowseResultsRenderer"]["tabs"][0]["tabRenderer"]["content"]);
            // trace!("Array Path Trace: {:#?}", value["contents"]["twoColumnBrowseResultsRenderer"]["tabs"][0]["tabRenderer"]["content"]["sectionListRenderer"]);
            // trace!("Array Path Trace: {:#?}", value["contents"]["twoColumnBrowseResultsRenderer"]["tabs"][0]["tabRenderer"]["content"]["sectionListRenderer"]["contents"]);
            // trace!("Array Path Trace: {:#?}", value["contents"]["twoColumnBrowseResultsRenderer"]["tabs"][0]["tabRenderer"]["content"]["sectionListRenderer"]["contents"][0]);
            // trace!("Array Path Trace: {:#?}", value["contents"]["twoColumnBrowseResultsRenderer"]["tabs"][0]["tabRenderer"]["content"]["sectionListRenderer"]["contents"][0]["playlistVideoListRenderer"]);
            // trace!("Array Path Trace: {:#?}", value["contents"]["twoColumnBrowseResultsRenderer"]["tabs"][0]["tabRenderer"]["content"]["sectionListRenderer"]["contents"][0]["playlistVideoListRenderer"]["contents"]);
            debug!("Getting first part of video array");
            value
                ["contents"]["twoColumnBrowseResultsRenderer"]["tabs"][0]           // Useless Array
                ["tabRenderer"]["content"]["sectionListRenderer"]["contents"][0]    // Weird continuation token
                ["itemSectionRenderer"]["contents"][0]                              // Useless Array
                ["playlistVideoListRenderer"]["contents"]                           // Video index in playlist (Last index is the continuation token)
                .as_array().expect("Failed to find video array")                    // To Array and unwrap
            }
        false => {
            // trace!("Array Path Trace: {:#?}", value);
            // trace!("Array Path Trace: {:#?}", value["onResponseReceivedActions"]);
            // trace!("Array Path Trace: {:#?}", value["onResponseReceivedActions"][0]);
            // trace!("Array Path Trace: {:#?}", value["onResponseReceivedActions"][0]["appendContinuationItemsAction"]);
            // trace!("Array Path Trace: {:#?}", value["onResponseReceivedActions"][0]["appendContinuationItemsAction"]["continuationItems"]);
            debug!("Getting n part of video array");
            value
                ["onResponseReceivedActions"][0]                                    // Useless Array
                ["appendContinuationItemsAction"]["continuationItems"]              // Video Array
                .as_array().expect("Failed to find video array")                    // To Array and unwrap
            }
        }
    }
    async fn get_videos(playlist_id: &str, continuation_token: Option<&str>, playlist_title: &str, playlist_path: &Path) -> Vec<Arc<Video>> {
        let mut videos = vec![];
        let (value, first) = match continuation_token {
            None => {
                (Self::get_value(&playlist_id, None).await, true)
            }
            Some(continuation_token) => {
                (Self::get_value(&playlist_id, Some(continuation_token)).await, false)
            }
        };

        for value in Self::get_array(&value, first) {
            let video = Video::get_video(value, &playlist_title, playlist_path);
            if video.id == "null" {
                //TODO: This is really dumb, it could be null for plenty of reasons other then we
                //      ran out of videos in the array... :O
                debug!("Video id == null (end of array probably)");
                let token = Self::get_continuation_token(&value);
                let buf = Box::pin(Self::get_videos(playlist_id, Some(&token), playlist_title, playlist_path)).await;
                videos.extend(buf);
                continue;
            }
            videos.push(video.into());
        }

        return videos;
    }
}

impl Video {
    fn get_video(value: &Value, playlist_title: &str, playlist_path: &Path) -> Self {
        let bloat = &value["playlistVideoRenderer"];
        let title = bloat["title"]["runs"][0]["text"].to_string().replace("\"", "");
        trace!("Video title: {title}");
        let formatted_title = format!("{} - {}.opus", playlist_title,
                    title
                        .replace("|", "｜")
                        .replace(":", "：") 
                        .replace("\\", "＂")
                        .replace("/", "⧸")
                        .replace("?", "？")
        );
        trace!("Video formatted title: {title}");
        let author = bloat["shortBylineText"]["runs"][0]["text"].to_string().replace("\"", "");
        trace!("Video author: {author}");
        let id = bloat["videoId"].to_string().replace("\"", "");
        trace!("Video id: {id}");
        let path = playlist_path.to_owned().join(&formatted_title);
        trace!("Video path: {}", path.display());
        Self {
            title,
            formatted_title,
            author,
            id,
            path,
        }
    }
}
