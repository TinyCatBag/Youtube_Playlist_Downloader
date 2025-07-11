use std::{fs::read_to_string};
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
    /// Add a cookies file if you want to download age-restriced videos.
    /// The yt-dlp github has a good guide on it, I recommend using the private window method with a new account.
    /// https://github.com/yt-dlp/yt-dlp/wiki/Extractors#exporting-youtube-cookies
    #[arg(short, long, verbatim_doc_comment)]
    cookies: Option<String>
}


#[tokio::main]
async fn  main() {

    let args = Args::parse();

    if args.file.is_none() && args.id.is_none(){
        println!("No ID nor File with IDs was provided :3");
        return
    }

    if let Some(path) = args.file {
        let file = read_to_string(path).unwrap();
        let lines = file.lines();

        let mut handles = tokio::task::JoinSet::new();
        for id in lines {
            let id = id.to_owned();
            let cookies = args.cookies.to_owned();
            handles.spawn(async move { 
                // sleep(Duration::from_millis(500));
                let download_request = DownloadRequest::check_playlist(&id).await;
                if cookies.is_some() {
                    download_request.download_playlist(cookies).await;
                }
                else {
                    download_request.download_playlist(None).await;
                }
                download_request.remove_vidoes().await;
            });
        }
        handles.join_all().await;
    }

    if let Some(id) = args.id {
        let download_request = DownloadRequest::check_playlist(&id).await;
        if args.cookies.is_some() {
            download_request.download_playlist(args.cookies).await;
        }
        else {
            download_request.download_playlist(None).await;
        }
        download_request.remove_vidoes().await;
    }
}
