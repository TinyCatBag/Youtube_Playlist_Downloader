use std::collections::{HashMap};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::usize;


use log::debug;
use serde::{Deserialize, Serialize};

use crate::{downloader::DownloadRequest, scraping::*};
use crate::name::NameWhole;

#[derive(Serialize, Deserialize)]
pub struct LocalPlaylist {
    // location: PathBuf,
    pub playlist: Playlist,
    pub playlist_hashmap: HashMap<String, usize>, //Video ID and index in playlist
    pub download_dir: PathBuf,
    pub download_name: NameWhole,
}

impl LocalPlaylist {
    pub async fn new(create: String, download_name: Option<&str>, download_dir: Option<String>) -> Self{
        // y + p will prevail
        if create.is_empty() {panic!("Arguments cannot be empty")}
        //first is the id second is the path

        let download_dir = DownloadRequest::string_to_download_directory(download_dir);
        let download_name = NameWhole::from_string(&download_name);

        let playlist = Playlist::new_no_directory(create, download_dir.clone()).await;
        let playlist_hashmap = playlist.make_playlist_hashmap_with_indexes();
        Self {
            playlist,
            playlist_hashmap,
            download_dir,
            download_name,
        }
    }
    /// Include the path with the filename
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self{
        let path = path.as_ref();
        //TODO: ERROR HANDLING
        let file = File::open(path).expect("Failed to open playlist file!");
        serde_json::from_reader::<_, Self>(file).expect("Failed to convert file into playlist")
    }
    /// Include the path with the filename
    ///
    /// Default is download_dir/playlist_title
    //TODO: I cant be damned to the the P as above
    pub fn into_file(self: &Self, path: &PathBuf) {
        //TODO: ERROR HANDLING
        debug!("Creating file: {}", path.display());
        let mut file = File::create(path).expect("Failed to open playlist file!");
        let json = serde_json::to_string(self).expect("Failed to make json out of playlist");
        file.write_all(json.as_bytes()).expect("Failed to write to file");
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
    //TODO: update the playlist_hashmap
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
    pub fn remove_video(self: &mut Self, video: Video) {
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
    pub fn into_download_request(self: Self) -> DownloadRequest {
        let playlist_hashmap = self.playlist.make_playlist_hashmap_with_videos();
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
    pub fn update_download_dir(self: &mut Self, download_dir: &String) {
        let download_dir = DownloadRequest::string_to_download_directory(Some(download_dir.to_string()));
        self.playlist.path = download_dir.clone();
        self.download_dir = download_dir;
    }
    pub fn update_download_name(self: &mut Self, download_name: &String) {
        self.download_name = NameWhole::from_string(&Some(download_name));
    }

}

#[cfg(test)]
mod tests {
    use std::env::current_dir;

    use super::*;
    #[tokio::test]
    async fn add_repeat_playlist_to_playlist() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).is_test(true).try_init();
        let mut local_playlist = LocalPlaylist::new("PLsRDS81Gj7jjWhOkM2IqSHfh2kLNadO2U".to_owned(), Some(&"Testing"), None).await;
        local_playlist.add_playlist(Playlist::new_no_directory("PLsRDS81Gj7jjWhOkM2IqSHfh2kLNadO2U".to_owned(), current_dir().unwrap().join("Testing")).await);
        assert_eq!(local_playlist.playlist.videos.len(), 1)
    }
    #[tokio::test]
    async fn add_playlist_to_playlist() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).is_test(true).try_init();
        let mut local_playlist = LocalPlaylist::new("PLsRDS81Gj7jjWhOkM2IqSHfh2kLNadO2U".to_owned(), Some(&"Testing"), None).await;
        let extra_playlist = Playlist::new("PLDVAy0zxsuUXZ_z5GNw9iUVaCStArbSDJ".to_owned(), current_dir().unwrap().join("Testing")).await;
        let extra_len = extra_playlist.videos.len();
        local_playlist.add_playlist(extra_playlist);
        assert_eq!(local_playlist.playlist.videos.len(), 1+extra_len)
    }
    #[tokio::test]
    async fn add_repeat_video_to_playlist() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).is_test(true).try_init();
        let mut local_playlist = LocalPlaylist::new("PLsRDS81Gj7jjWhOkM2IqSHfh2kLNadO2U".to_owned(), Some(&"Testing"), None).await;
        local_playlist.add_video(Video { title: "Nightcore - Not The One (Lyrics)".to_owned(), author: "RubyChan's Nightcore".to_owned(), id: "11WbMT_IBrY".to_owned(), path: current_dir().unwrap() });
        assert_eq!(local_playlist.playlist.videos.len(), 1)
    }
    #[tokio::test]
    async fn add_video_to_playlist() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).is_test(true).try_init();
        let mut local_playlist = LocalPlaylist::new("PLsRDS81Gj7jjWhOkM2IqSHfh2kLNadO2U".to_owned(), Some(&"Testing"), None).await;
        local_playlist.add_video(Video { title: "Nightcore - Not The One (Lyrics) Not A Repeat".to_owned(), author: "RubyChan's Nightcore".to_owned(), id: "11WbMT_IBrY Not The Same ID".to_owned(), path: current_dir().unwrap() });
        assert_eq!(local_playlist.playlist.videos.len(), 2)
    }
    #[tokio::test]
    async fn remove_nonexistent_playlist_from_playlist() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).is_test(true).try_init();
        let mut local_playlist = LocalPlaylist::new("PLsRDS81Gj7jjWhOkM2IqSHfh2kLNadO2U".to_owned(), Some(&"Testing"), None).await;
        local_playlist.remove_playlist(Playlist::new_no_directory("PLDVAy0zxsuUXZ_z5GNw9iUVaCStArbSDJ", current_dir().unwrap().join("Testing")).await);
        assert_eq!(local_playlist.playlist.videos.len(), 1)
    }
    #[tokio::test]
    async fn remove_playlist_from_playlist() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).is_test(true).try_init();
        let mut local_playlist = LocalPlaylist::new("PLsRDS81Gj7jjWhOkM2IqSHfh2kLNadO2U".to_owned(), Some(&"Testing"), None).await;
        local_playlist.remove_playlist(Playlist::new_no_directory("PLsRDS81Gj7jjWhOkM2IqSHfh2kLNadO2U", current_dir().unwrap().join("Testing")).await);
        assert_eq!(local_playlist.playlist.videos.len(), 0)
    }
    #[tokio::test]
    async fn remove_nonexisitent_video_from_playlist() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).is_test(true).try_init();
        let mut local_playlist = LocalPlaylist::new("PLsRDS81Gj7jjWhOkM2IqSHfh2kLNadO2U".to_owned(), Some(&"Testing"), None).await;
        local_playlist.remove_video(Video { title: "Nightcore - Not The One (Lyrics) Not A Repeat".to_owned(), author: "RubyChan's Nightcore".to_owned(), id: "11WbMT_IBrY Not The Same ID".to_owned(), path: current_dir().unwrap() });
        assert_eq!(local_playlist.playlist.videos.len(), 1)
    }
    #[tokio::test]
    async fn remove_video_from_playlist() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).is_test(true).try_init();
        let mut local_playlist = LocalPlaylist::new("PLsRDS81Gj7jjWhOkM2IqSHfh2kLNadO2U".to_owned(), Some(&"Testing"), None).await;
        local_playlist.remove_video(Video { title: "Nightcore - Not The One (Lyrics)".to_owned(), author: "RubyChan's Nightcore".to_owned(), id: "11WbMT_IBrY".to_owned(), path: current_dir().unwrap() });
        assert_eq!(local_playlist.playlist.videos.len(), 0)
    }
    #[tokio::test]
    async fn write_and_read_local_file() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).is_test(true).try_init();
        let local_playlist = LocalPlaylist::new("PLsRDS81Gj7jjWhOkM2IqSHfh2kLNadO2U".to_owned(), Some(&"Testing"), None).await;
        local_playlist.into_file(&current_dir().unwrap().join("Test.json"));
        let file_playlist = LocalPlaylist::from_file(&current_dir().unwrap().join("Test.json").as_path());
        assert_eq!(local_playlist.playlist, file_playlist.playlist)
    }
}
