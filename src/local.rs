use std::collections::{hash_map, HashMap};
use std::fs::File;
use std::path::{PathBuf};
use std::sync::Arc;
use std::usize;

use log::{debug, trace};
use opusmeta::{LowercaseString, Tag};
use serde::{Deserialize, Serialize};
use tokio::io::unix::AsyncFdTryNewError;

use crate::{downloader::DownloadRequest, scraping::*};
use crate::name::NameWhole;

#[derive(Serialize, Deserialize)]
struct LocalPlaylist {
    playlist: Playlist,
    playlist_hashmap: HashMap<String, usize>, //Video ID and index in playlist
    download_dir: PathBuf,
    download_name: NameWhole
}

impl LocalPlaylist {
    fn from_download_request(download_request: DownloadRequest) -> Self {
        let playlist_hashmap = {
            let mut hashmap = HashMap::new();
            for i in 0..download_request.playlist.videos.len(){
                hashmap.insert(download_request.playlist.videos[i].id.clone(), i);
            }
            hashmap
        };
        LocalPlaylist { playlist: download_request.playlist, download_dir: download_request.download_dir, download_name: download_request.download_name, playlist_hashmap}
    }
    fn from_file(path: &PathBuf) -> Self{
        //TODO: ERROR HANDLING
        let file = File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    }
    fn add_playlist(self: &mut Self, playlist: Playlist) {
        for video in &playlist.videos {
            if !self.playlist_hashmap.contains_key(&video.id){
                self.playlist.videos.push(video.clone());
            }
        }
    }
    fn add_video(self: &mut Self, video: Video) {
        if !self.playlist_hashmap.contains_key(&video.id){
            self.playlist.videos.push(Arc::new(video));
        }
    }
    fn remove_playlist(self: &mut Self, playlist: &Playlist) {
        let mut local_hashmap: HashMap<&String, usize> = HashMap::new();
        let mut indexes = Vec::new();
        for i in 0..self.playlist.videos.len() {
            local_hashmap.insert(&self.playlist.videos[i].id, i);
        }
        for video in &playlist.videos {
            match local_hashmap.get(&video.id) {
                Some(i) => {
                    indexes.push(*i);
                },
                None => continue,
            }
        }
        indexes.sort();
        let mut offset = 0;
        for i in indexes {
            self.playlist.videos.remove(i-offset);
            offset += 1
        }
    }
    fn remove_video(self: &mut Self, video: &Video) {
        for i in 0..self.playlist.videos.len() {
            if &self.playlist.videos[i].id == &video.id {
                self.playlist.videos.remove(i);
                return
            }
        }
    }
    fn list_playlist(self: &Self) {
        println!("Playlist: {}", self.playlist.title);
        println!("Download path: {}", self.download_dir.to_str().unwrap());
        //TODO better writing
        println!("Name schema: {:#?}", self.download_name);
        for video in &self.playlist.videos {
            println!("Title: {}, Author: {}", video.title, video.author)
        }
    }
    //TODO: this is just repeating code ffs
    fn into_download_request(self: Self) -> DownloadRequest {
        let playlist_hashmap = {
            debug!("Making playlist hashmap");
            let mut hashmap = HashMap::new();
            for video in &self.playlist.videos {
                hashmap.insert(video.id.clone(), video.clone());
                trace!("Video id: {}", video.id);
            }
            hashmap
        };
        let directory_hashmap = {
            debug!("Making directory hashmap");
            let mut hashmap = HashMap::new();
            for entry in self.download_dir.read_dir().unwrap() {
                let entry = entry.unwrap();
                trace!("entry: {}", entry.file_name().display());
                if entry.file_type().unwrap().is_file() { trace!("Skipped file: {}", entry.file_name().display()); continue; }
                if entry.file_name().to_str().unwrap() == self.playlist.title {
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
                        
                        let title = tag.get_one(&LowercaseString::new("Title")).unwrap().to_owned();
                        // let formatted_title = format!("{} - {}", playlist.title, title);
                        let author = tag.get_one(&LowercaseString::new("Artist")).unwrap().to_owned();
                        let id = tag.get_one(&LowercaseString::new("Video_ID")).unwrap().to_owned();
                        // trace!("Metadata: title: {title}, formatted_title: {formatted_title}, author: {author}, id: {id}");
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
        };
        let missing_videos = {
            let mut hashmap = HashMap::new();
            for (video_id, video) in playlist_hashmap.clone() {
                if !directory_hashmap.contains_key(&video_id) && video.id != "Legacy"{
                    debug!("Added video to missing queue: id: {}, path: {}, title: {}", video.id, video.path.display(), video.title);
                    hashmap.insert(video_id, video);
                }
            }
            hashmap
        };
        let remove_vidoes = {
            let mut hashmap = HashMap::new();
            for (video_id, video) in directory_hashmap {
                if !playlist_hashmap.contains_key(&video_id) && video.id != "Legacy"{
                    debug!("Added video to remove queue: id: {}, path: {}, title: {}", video.id, video.path.display(), video.title);
                    hashmap.insert(video_id, video);
                }
            }
            hashmap
        };
        DownloadRequest {
            download_dir: self.download_dir,
            download_name: self.download_name,
            playlist: self.playlist,
            missing_videos,
            remove_vidoes,
        }
    }
}
