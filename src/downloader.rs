use std::{collections::HashMap, env::current_dir, fs::create_dir_all, path::PathBuf, process::Command, sync::Arc};
use log::{debug, trace, warn, info};
use opusmeta::{LowercaseString, Tag};
//TODO: implement idicatif again
//TODO: Still have to change the formatted title to match the file names tho that will be easy

pub mod scraping;
use scraping::*;

pub mod name;
use name::*;

pub struct DownloadRequest {
    pub download_dir: PathBuf,
    pub download_name: NameWhole,
    pub playlist: Playlist,
    pub missing_videos: HashMap<String, Arc<Video>>,
    pub removed_vidoes: HashMap<String, Video>,
}


impl DownloadRequest {
    pub async fn check_playlist(id: impl AsRef<str>, download_dir: Option<String>, download_name: Option<&str>) -> Self {
        let download_dir = Self::string_to_download_directory(download_dir);
        let download_name = NameWhole::from_string(&download_name);

        let id = id.as_ref();
        let playlist = Playlist::new(id, download_dir.clone()).await;

        let playlist_hashmap = playlist.make_playlist_hashmap_with_videos();
        let directory_hashmap = Self::make_directory_hashmap(&download_dir, &playlist.title);
        let missing_videos = Self::missing_videos(&playlist_hashmap, &directory_hashmap);
        let removed_vidoes = Self::removed_videos(&playlist_hashmap, directory_hashmap);

        Self {
            download_dir,
            download_name,
            playlist,
            // playlist_hashmap,
            // directory_hashmap,
            missing_videos,
            removed_vidoes,
        }
    }
        
    pub async fn download_playlist(self: &Self, cookies: Option<String>) {
        let mut counter = 1;
        for (video_id, video) in &self.missing_videos {
            let formatted_title = self.download_name.formatted_video_title(video, &self.playlist);
            let download_path = PathBuf::from(self.download_name.formatted_download_path(video, &self.playlist));
            info!("Downloading: {} | {} out of {}", formatted_title, counter, self.missing_videos.len());
            let mut args = vec!["--embed-thumbnail".to_string(), 
                    "-x".to_string() ,"--audio-format".to_string(), "opus".to_string(),
                    "-o".to_string(), format!("{0}", download_path.display()),
                    format!("https://www.youtube.com/watch?v={}", video.id)];
                if let Some(cookies) = &cookies {
                    args.push("--cookies".to_string());
                    args.push(cookies.to_string());
                }
            debug!("Downloading video to: {}", download_path.display());
            let mut downloader = Command::new("yt-dlp")
                .args(args)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .spawn().unwrap();

            downloader.wait().expect("Downloader failed =3");

            match Tag::read_from_path(&download_path) {
                Ok(mut tag) => {
                    tag.add_one(LowercaseString::new("Title"), video.title.clone());
                    tag.add_one(LowercaseString::new("Artist"), video.author.clone());
                    tag.add_one(LowercaseString::new("Performer"), video.author.clone());
                    tag.add_one(LowercaseString::new("Video_ID"), video.id.clone());
                    tag.add_one(LowercaseString::new("Album"), self.playlist.title.clone());
                    tag.write_to_path(&download_path).unwrap();
                },
                // Cant get the error kiind ffs
                //TODO: Read the docs about this
                Err(_) => {
                    if !download_path.exists() {
                        warn!("Failed to download: {}, at {}", video_id, download_path.display());
                        warn!("You should check if the video is age-restricted, if it is you need to pass cookies, to learn about that just do -h.");
                        warn!("Otherwise please report it on my github!")
                    }
                    else {
                        warn!("Failed to read tag from: {}, at {}", video_id, download_path.display());
                        warn!("You should remove the file as it will have no tags and the program will try to download it again.");
                        //TODO: do this for the user :L
                    }
                },
            };
            info!("Finished downloading: {} | {} out of {}", formatted_title, counter, self.missing_videos.len());
            counter+=1;
        }
    }

    pub async fn remove_vidoes(self: &Self) {
        for (_video_id, video) in &self.removed_vidoes {
            debug!("Video found in remove queue: title: {}, path: {}, id: {}", video.title, &video.path.display(), video.id)
        }
    }

    pub fn string_to_download_directory(string: Option<String>) -> PathBuf {
        match string {
            Some(path) => {
                //Clap does in fact allow empty string's with "", i love life!
                if path.is_empty() {
                    warn!("Empty output path, using current directory! (Default)");
                    let dir = current_dir().unwrap();
                    if let Err(error) = create_dir_all(&dir){
                        println!("{:#?}, kind {:#?}", error, error.kind())
                    };
                    dir
                }
                else if path.chars().nth(0).unwrap() == '/' {
                    debug!("Using absolute path!");
                    let dir = PathBuf::from(path);
                    if let Err(error) = create_dir_all(&dir){
                        println!("{:#?}, kind {:#?}", error, error.kind())
                    };
                    dir
                }
                else {
                    debug!("Adding output directory to current directory");
                    let dir = current_dir().unwrap().join(PathBuf::from(&path));
                    if let Err(error) = create_dir_all(&dir){
                        println!("{:#?}, kind {:#?}", error, error.kind())
                    };
                    dir
                }
            },
            None => {
                debug!("No path provided, using current directory (Default)");
                current_dir().unwrap()
            },
        }
    }
    //String is a video ID
    pub fn make_directory_hashmap(download_dir: &PathBuf, title: &String) -> HashMap<String, Video>{
        debug!("Making directory hashmap");
        let mut hashmap = HashMap::new();
        for entry in download_dir.read_dir().unwrap() {
            let entry = entry.unwrap();
            trace!("entry: {}", entry.file_name().display());
            if entry.file_type().unwrap().is_file() { trace!("Skipped file: {}", entry.file_name().display()); continue; }
            if entry.file_name().to_str().unwrap() == title {
                debug!("Found directory: {}", entry.file_name().display());
                for inner_entry in entry.path().read_dir().unwrap() {
                    let inner_entry = inner_entry.unwrap();
                    trace!("Inner entry: {}", inner_entry.file_name().display());
                    if inner_entry.file_type().unwrap().is_dir() { trace!("Skipped file: {}", inner_entry.file_name().display()); continue; }

                    let path = inner_entry.path();
                    trace!("Inner entry path: {}", path.display());
                    let tag = Tag::read_from_path(&path).unwrap();

                    //Gave info to the user 
                    //TODO: if i can figure out how to not have to panic here without just skipping it!
                    let title = tag.get_one(&LowercaseString::new("Title"));
                    let author = tag.get_one(&LowercaseString::new("Artist"));
                    let id = tag.get_one(&LowercaseString::new("Video_ID"));
                    let (title, author, id) = match (title, author, id) {
                        (Some(title), Some(author), Some(id)) => {
                            (title.to_owned(), author.to_owned(), id.to_owned())
                        }
                        _ => {
                            panic!("File is missing metadata, please remove it or move it! File path:{}", path.display())
                        }
                    };

                    trace!("Metadata: title: {title}, author: {author}, id: {id}");
                    let video = Video {
                        title,
                        // formatted_title,
                        author,
                        id: id.clone(),
                        path,
                    };
                    hashmap.insert(id, video);
                }
            }
        }
        hashmap
    }

    //String is the video ID
    pub fn missing_videos(playlist_hashmap: &HashMap<String, Arc<Video>>, directory_hashmap: &HashMap<String, Video>) -> HashMap<String, Arc<Video>> {
        let mut hashmap = HashMap::new();
        for (video_id, video) in playlist_hashmap.clone() {
            if !directory_hashmap.contains_key(&video_id) && video.id != "Legacy"{
                debug!("Added video to missing queue: id: {}, path: {}, title: {}", video.id, video.path.display(), video.title);
                hashmap.insert(video_id, video);
            }
        }
        hashmap
    }

    pub fn removed_videos(playlist_hashmap: &HashMap<String, Arc<Video>>, directory_hashmap: HashMap<String, Video>) -> HashMap<String, Video> {
        let mut hashmap = HashMap::new();
        for (video_id, video) in directory_hashmap {
            if !playlist_hashmap.contains_key(&video_id) && video.id != "Legacy"{
                debug!("Added video to remove queue: id: {}, path: {}, title: {}", video.id, video.path.display(), video.title);
                hashmap.insert(video_id, video);
            }
        }
        hashmap
    }
}
