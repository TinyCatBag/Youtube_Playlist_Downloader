use std::{fs::read_to_string};
use log::{debug};
use env_logger::Env;
use clap::{command, Parser, Subcommand};

mod downloader;
use downloader::*;

mod local;
use local::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Output directory can be absolute. Case sensitive!
    #[arg(short, long)]
    output: Option<String>,
    /// Name for file that will be downloaded.
    /// You can put in video/playlist specific things in the name for e.g.
    /// "{PlaylistTitle} - {VideoTitle}" will be converted to "CoolPlaylist - Really cool song.opus".
    /// Variables that can be accesed by putting them in {}
    /// VideoTitle,     PlaylistTitle,
    /// CurrentDate,    ReleaseDate,
    /// Author,         VideoID,
    /// You can still include '{' and '}' in the file name by typing them twice to breakout of them!
    #[arg(short, long, verbatim_doc_comment)]
    name: Option<String>,
    /// Add a cookies file if you want to download age-restriced videos.
    /// The yt-dlp github has a good guide on it, I recommend using the private window method with a new account.
    /// https://github.com/yt-dlp/yt-dlp/wiki/Extractors#exporting-youtube-cookies
    #[arg(short, long, verbatim_doc_comment)]
    cookies: Option<String>,
    //TODO: This can be local/remote which would make more sense think of a better name tho!
    #[command(subcommand)]
    target: TargetCommands,
}

#[derive(Subcommand, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum TargetCommands {
    /// Allows you to make and edit your playlists localy without having to go through youtube,
    /// it will not reflect on the actual playlist in the cloud yet!!!
    #[command(verbatim_doc_comment)]
    Local {
        /// Add video/videos to a playlist (if adding from a playlist will ignore duplicates)
        #[arg(short, long, value_parser = parse_video_or_playlist)]
        add: Option<VideoOrPlaylist>,
        /// Remvove video/videos from a playlist.
        #[arg(short, long, value_parser = parse_video_or_playlist)]
        remove: Option<VideoOrPlaylist>,
        /// Downlaod a playlist.
        #[arg(short, long)]
        download: Option<String>,
        /// Lists all videos in a locally saved playlist.
        #[arg(short, long)]
        list: Option<String>,
    },
    /// Download playlists from youtube
    Remote{
        /// Downlaod a playlist.
        #[arg(short, long)]
        download: Option<String>,
        /// Playlist ID to download.
        #[arg(short, long)]
        id: Option<String>,
        /// File to read ID's from.
        #[arg(short, long)]
        file: Option<String>,
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum VideoOrPlaylist {
    Video(String),
    Playlist(String)
}

fn parse_video_or_playlist(str: &str) -> Result<VideoOrPlaylist, String>{
    if str.to_lowercase().starts_with("video:") {
        Ok(VideoOrPlaylist::Video(str[5..].to_string()))
    }
    else if str.to_lowercase().starts_with("playlist:") {
        Ok(VideoOrPlaylist::Playlist(str[9..].to_string()))
    }
    else {
        Err(format!("String not recognised! Please use video:{{Video_ID}} or playlist:{{Playlist_ID}}\nString: {}", str))
    }
}

#[tokio::main]
async fn  main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    match args.target {
        TargetCommands::Local { add, remove, download, list } => {
            todo!("Didnt make any local things yet!")
        }
        //TOOD: The code below is complete and utter dogshit(Again), maybe actually idk=:3
        TargetCommands::Remote { download, id, file } => {
            if file.is_none() && id.is_none(){
                panic!("No ID nor File with IDs was provided :3");
            }

            if let Some(path) = file {
                debug!("File path: {path}");
                let file = read_to_string(path).unwrap();
                let lines = file.lines();

                let mut handles = tokio::task::JoinSet::new();
                for id in lines {
                    let id = id.to_owned();
                    let output = args.output.to_owned();
                    let name = args.name.to_owned();
                    let cookies = args.cookies.to_owned();
                    debug!("ID: {id}");
                    debug!("Output: {:#?}", output);
                    debug!("Name: {:#?}", name);
                    debug!("Cookies: {:?}", cookies);
                    handles.spawn(async move { 
                        debug!("Making DownloadRequest");
                        let download_request = DownloadRequest::check_playlist(
                            &id,
                            output,
                            name.as_deref()
                        ).await;
                        if cookies.is_some() {
                            debug!("Downloading Playlist with cookies");
                            download_request.download_playlist(cookies).await;
                        }
                        else {
                            debug!("Downloading Playlist without cookies");
                            download_request.download_playlist(None).await;
                        }
                        debug!("Removing videos removed by user");
                        download_request.remove_vidoes().await;
                    });
                }
                debug!("Joining handles");
                handles.join_all().await;
            }

            if let Some(id) = id {
                debug!("ID: {id}");
                debug!("Output: {:?}", args.output);
                debug!("Name: {:?}", args.name);
                debug!("Cookies: {:?}", args.cookies);

                debug!("Making DownloadRequest");
                let download_request = DownloadRequest::check_playlist(
                    &id,
                    args.output,
                    args.name.as_deref()
                ).await;
                if args.cookies.is_some() {
                    debug!("Downloading Playlist with cookies");
                    download_request.download_playlist(args.cookies).await;
                }
                else {
                    debug!("Downloading Playlist without cookies");
                    download_request.download_playlist(None).await;
                }
                debug!("Removing videos removed by user");
                download_request.remove_vidoes().await;
            }
        }
    }
}
