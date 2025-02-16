use tokio;
use clap::Parser;

mod library;
use library::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Playlist ID to download
    #[arg(short, long)]
    id: Option<String>,
    /// File to read from
    #[arg(short, long)]
    file: Option<String>,
}


#[tokio::main]
async fn  main() {
    let args = Args::parse();
    
    if args.file.is_none() && args.id.is_none(){
        println!("No ID nor File with IDs was provided :3");
        return
    }

    if args.file.is_some() {
        let mut handles = tokio::task::JoinSet::new();
        let playlist_ids = read_file(args.file.unwrap());
        for x in playlist_ids{
            handles.spawn(async move { 
                download_playlist(&x[..]).await;
            });
        }
        handles.join_all().await;
    }

    if args.id.is_some() {
        download_playlist(&args.id.unwrap()[..]).await;
    }
}
