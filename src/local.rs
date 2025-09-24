use std::collections::{HashMap};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::usize;
use std::error::Error;


use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{downloader::DownloadRequest, scraping::*};
use crate::name::NameWhole;

#[derive(Serialize, Deserialize)]
pub struct LocalPlaylist {
    // location: PathBuf,
    playlist: Playlist,
    playlist_hashmap: HashMap<String, usize>, //Video ID and index in playlist
    download_dir: PathBuf,
    download_name: NameWhole
}

impl LocalPlaylist {
    pub fn from_download_request(download_request: DownloadRequest) -> Self {
        let playlist_hashmap = {
            let mut hashmap = HashMap::new();
            for i in 0..download_request.playlist.videos.len(){
                hashmap.insert(download_request.playlist.videos[i].id.clone(), i);
            }
            hashmap
        };
        LocalPlaylist { playlist: download_request.playlist, download_dir: download_request.download_dir, download_name: download_request.download_name, playlist_hashmap}
    }
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>>{
        let path = path.as_ref();
        //TODO: ERROR HANDLING
        let file = File::open(path)?;
        Ok(serde_json::from_reader::<_, Self>(file)?)
    }
    pub fn add_playlist(self: &mut Self, playlist: Playlist) {
        for video in &playlist.videos {
            if !self.playlist_hashmap.contains_key(&video.id){
                self.playlist.videos.push(video.clone());
            }
        }
    }
    pub fn add_video(self: &mut Self, video: Video) {
        if !self.playlist_hashmap.contains_key(&video.id){
            self.playlist.videos.push(Arc::new(video));
        }
    }
    pub fn remove_playlist(self: &mut Self, playlist: Playlist) {
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
    pub fn remove_video(self: &mut Self, video: &Video) {
        for i in 0..self.playlist.videos.len() {
            if &self.playlist.videos[i].id == &video.id {
                self.playlist.videos.remove(i);
                return
            }
        }
    }
    pub fn list_playlist(self: &Self) {
        println!("Playlist: {}", self.playlist.title);
        println!("Download path: {}", self.download_dir.to_str().unwrap());
        //TODO better writing
        println!("Name schema: {:#?}", self.download_name);
        for video in &self.playlist.videos {
            println!("Title: {}, Author: {}", video.title, video.author)
        }
    }
    //TODO: this is just repeating code ffs
    pub fn into_download_request(self: Self) -> DownloadRequest {
        let playlist_hashmap = self.playlist.make_playlist_hashmap();
        let directory_hashmap = DownloadRequest::make_directory_hashmap(&self.download_dir, &self.playlist.title);
        let missing_videos = DownloadRequest::missing_videos(&playlist_hashmap, &directory_hashmap);
        let removed_vidoes = DownloadRequest::removed_videos(&playlist_hashmap, directory_hashmap);

        DownloadRequest {
            download_dir: self.download_dir,
            download_name: self.download_name,
            playlist: self.playlist,
            missing_videos,
            removed_vidoes,
        }
    }
}
