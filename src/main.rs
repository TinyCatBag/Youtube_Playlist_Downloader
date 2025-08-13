use std::{fs::read_to_string};
use log::{debug};
use env_logger::Env;
use clap::Parser;
mod downloader;
use downloader::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Playlist ID to download
    #[arg(short, long)]
    id: Option<String>,
    /// File to read from
    #[arg(short, long)]
    file: Option<String>,
    /// Output directory can be absolute. Case sensitive!
    #[arg(short, long)]
    output: Option<String>,
    /// Name for file that will be downloaded
    #[arg(short, long)]
    name: Option<String>,
    /// Add a cookies file if you want to download age-restriced videos.
    /// The yt-dlp github has a good guide on it, I recommend using the private window method with a new account.
    /// https://github.com/yt-dlp/yt-dlp/wiki/Extractors#exporting-youtube-cookies
    #[arg(short, long, verbatim_doc_comment)]
    cookies: Option<String>
}


#[tokio::main]
async fn  main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    if args.file.is_none() && args.id.is_none(){
        panic!("No ID nor File with IDs was provided :3");
    }

    if let Some(path) = args.file {
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
                    name
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

    if let Some(id) = args.id {
        debug!("ID: {id}");
        debug!("Output: {:?}", args.output);
        debug!("Name: {:?}", args.name);
        debug!("Cookies: {:?}", args.cookies);

        debug!("Making DownloadRequest");
        let download_request = DownloadRequest::check_playlist(
            &id,
            args.output,
            args.name
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
