use std::{collections::HashMap, env::current_dir, process::Command, sync::Arc};
use log::{debug, trace, warn, info};
use opusmeta::Tag;
//TODO: implement idicatif again

mod scraping;
use scraping::*;

pub struct DownloadRequest {
    playlist: Playlist,
    missing_videos:     HashMap<String, Arc<Video>>,
    remove_vidoes:      HashMap<String, Video>,
}

impl DownloadRequest {
    pub async fn check_playlist(id: impl AsRef<str>) -> Self {
        let id = id.as_ref();
        let playlist = Playlist::new(id).await;
        let playlist_hashmap = {
            debug!("Making playlist hashmap");
            let mut hashmap = HashMap::new();
            for video in &playlist.videos {
                hashmap.insert(video.id.clone(), video.clone());
                trace!("Video id: {}", video.id);
            }
            hashmap
        };
        let directory_hashmap = {
            debug!("Making directory hashmap");
            let mut hashmap = HashMap::new();
            for entry in current_dir().unwrap().read_dir().unwrap() {
                let entry = entry.unwrap();
                trace!("entry: {}", entry.file_name().display());
                if entry.file_type().unwrap().is_file() { trace!("Skipped file: {}", entry.file_name().display()); continue; }
                if entry.file_name().to_str().unwrap() == playlist.title {
                    debug!("Found directory: {}", entry.file_name().display());
                    for inner_entry in entry.path().read_dir().unwrap() {
                        let inner_entry = inner_entry.unwrap();
                        trace!("Inner entry: {}", inner_entry.file_name().display());
                        // println!("path_error: {:#?}", inner_entry.path());
                        if inner_entry.file_type().unwrap().is_dir() { trace!("Skipped file: {}", inner_entry.file_name().display()); continue; }

                        let path = inner_entry.path();
                        trace!("Inner entry path: {}", path.display());
                        let tag = Tag::read_from_path(&path).unwrap();
                        // trace!("Inner entry tags: {:?}", tag);

                        //TODO: if stopped midway this erros as the files have no metadata fuuuuck
                        //      implement error handling womp womp
                        
                        let title = tag.get_one("Title".to_string()).unwrap().to_owned();
                        let formatted_title = format!("{} - {}", playlist.title, title);
                        let author = tag.get_one("Artist".to_string()).unwrap().to_owned();
                        let id = tag.get_one("Video_ID".to_string()).unwrap().to_owned();
                        trace!("Metadata: title: {title}, formatted_title: {formatted_title}, author: {author}, id: {id}");
                        let video = Video {
                            title,
                            formatted_title,
                            author,
                            id: id.clone(),
                            path,
                        };
                        hashmap.insert(id, video);
                    }
                }
            }
            hashmap
        };
        let missing_videos = {
            let mut hashmap = HashMap::new();
            for (video_id, video) in playlist_hashmap.clone() {
                if !directory_hashmap.contains_key(&video_id) && video.id != "Legacy"{
                    debug!("Added video to missing queue: id: {}, path: {}, title: {}", video.id, video.path.display(), video.formatted_title);
                    hashmap.insert(video_id, video);
                }
            }
            hashmap
        };
        let remove_vidoes = {
            let mut hashmap = HashMap::new();
            for (video_id, video) in directory_hashmap {
                if !playlist_hashmap.contains_key(&video_id) && video.id != "Legacy"{
                    debug!("Added video to remove queue: id: {}, path: {}, title: {}", video.id, video.path.display(), video.formatted_title);
                    hashmap.insert(video_id, video);
                }
            }
            hashmap
        };
        Self {
            playlist,
            // playlist_hashmap,
            // directory_hashmap,
            missing_videos,
            remove_vidoes,
        }
    }
        
    pub async fn download_playlist(self: &Self, cookies: Option<String>) {
        let mut counter = 1;
        for (video_id, video) in &self.missing_videos {
            info!("Downloading: {} | {} out of {}", video.formatted_title, counter, self.missing_videos.len());
            // , "--cookies", cookies
            let mut args = vec!["--embed-thumbnail".to_string(), 
                    "-x".to_string() ,"--audio-format".to_string(), "opus".to_string(),
                    "-o".to_string(), format!("{0}/{0} - %(title)s.%(ext)s", self.playlist.title,),
                    format!("https://www.youtube.com/watch?v={}", video.id)];
                if let Some(cookies) = &cookies {
                    args.push("--cookies".to_string());
                    args.push(cookies.to_string());
                }
            let mut downloader = Command::new("yt-dlp")
                .args(args)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .spawn().unwrap();

            downloader.wait().expect("Downloader failed =3");

            match Tag::read_from_path(&video.path) {
                Ok(mut tag) => {
                    tag.add_one("Title".to_string(), video.title.clone());
                    tag.add_one("Artist".to_string(), video.author.clone());
                    tag.add_one("Performer".to_string(), video.author.clone());
                    tag.add_one("Video_ID".to_string(), video.id.clone());
                    tag.add_one("Album".to_string(), self.playlist.title.clone());
                    tag.write_to_path(&video.path).unwrap();
                },
                // Cant get the error kiind ffs
                //TODO: Read the docs about this
                Err(_) => {
                    if !video.path.exists() {
                        warn!("Failed to download: {}", video_id);
                        warn!("You should check if the video is age-restricted, if it is you need to pass cookies, to learn about that just do -h.");
                        warn!("Otherwise please report it on my github!")
                    }
                    else {
                        warn!("Failed to read tag from: {},", video_id);
                        warn!("You should remove the file as it will have no tags and the program will try to download it again.");
                        //TODO: do this for the user :L
                    }
                },
            };
            info!("Finished downloading: {} | {} out of {}", video.formatted_title, counter, self.missing_videos.len());
            counter+=1;
        }
    }

    pub async fn remove_vidoes(self: &Self) {
        for (_video_id, video) in &self.remove_vidoes {
            debug!("Video found in remove queue: title: {}, path: {}, id: {}", video.formatted_title, &video.path.display(), video.id)
        }
    }
}
