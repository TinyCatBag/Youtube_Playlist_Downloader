# Youtube_Playlist_Downloader
    Downloads all videos in a playlist/s as mp3 files.
    Files are downloaded into a directory with the title of the playlist.
----------
#### Youtube_Playlist_Downloader -i/--id *playlist_id*
    Repalce playlist_id with the id of the playlist.
    The id can be found in the url.
----------
#### Youtube_Playlist_Downloader -f/--file *file_with_ids*
    It will read all playlist id's from a file. The id's should be the only thing present and be seperated by newlines.
    For example:
        *PLH1TkRvPd30MSh23wIZTar8MUJ1w-XPB2*
        *PLH1TkRvPd30NvibNE3udXgIljcdXYuCPf*
        *PLH1TkRvPd30N9dIh6mkDx_X0g46r32ENf*
        *PLH1TkRvPd30M7hb9bnV2f1ZzN24c45vBF*
        *PLH1TkRvPd30OLQzSbKrV_crwq6FVOTFWh*
        *PLH1TkRvPd30Pa6QupChBNefWLREdW9Br9*
----------
#### Youtube_Playlist_Downloader -o/--output *output directory*
    Output directory for playlist. Can be absolute. It's case sensitive!
----------
#### Youtube_Playlist_Downloader -n/-name *custom name*
    You can put in video/playlist specific things in the name for e.g.
    "{PlaylistTitle} - {VideoTitle}" will be converted to "CoolPlaylist - Really cool song.opus".
    Variables that can be accesed by putting them in {}
        VideoTitle,     PlaylistTitle,
        CurrentDate,    ReleaseDate,
        Author,         VideoID,
----------
#### Youtube_Playlist_Downloader -c/-cookies *file_with_cookies*
    Obtain the file with your cookies by following the guide at
    https://github.com/yt-dlp/yt-dlp/wiki/Extractors#exporting-youtube-cookies
    I recommend using the private window method.
----------
