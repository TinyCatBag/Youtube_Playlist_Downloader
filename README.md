# Youtube_Playlist_Downloader
Downloads all videos in a playlist/s as opus files.  
Files are downloaded into a directory with the title of the playlist.  

---

## Remote Playlists
These are commands used for playlist hosted on YouTube.

---

### Youtube_Playlist_Downloader remote -i/--id *playlist_id*
Repalce playlist_id with the id of the playlist.  
The id can be found in the url.  

---

### Youtube_Playlist_Downloader remote -f/--file *file_with_ids*
It will read all playlist id's from a file. The id's should be the only thing present and be seperated by newlines.  
For example:  
> PLH1TkRvPd30MSh23wIZTar8MUJ1w-XPB2  
> PLH1TkRvPd30NvibNE3udXgIljcdXYuCPf  
> PLH1TkRvPd30N9dIh6mkDx_X0g46r32ENf  
> PLH1TkRvPd30M7hb9bnV2f1ZzN24c45vBF  
> PLH1TkRvPd30OLQzSbKrV_crwq6FVOTFWh  
> PLH1TkRvPd30Pa6QupChBNefWLREdW9Br9  

---

## Local Playlists
These are commands used for playlist stored as files on your PC.  

---

### Youtube_Playlist_Downloader local -c/--create *playlist_id*
Create a local playlist, argument is a playlist ID  
all options used with this such as -o/--output or -n/--name you can't
change them after creating a playlist yet... :P
E.g.  
```
    Youtube_Playlist_Downloader local -c PLH1TkRvPd30MSh23wIZTar8MUJ1w-XPB2
```
Would make a playlist from PLH1TkRvPd30MSh23wIZTar8MUJ1w-XPB2  
and put in a file called the same as the playlit's title.  

---

### Youtube_Playlist_Downloader local -t/--target *playlist_file*
Read a playlist from a .json file that -c creates.  
This option alone does nothing but is required for the ones below to function!  

---

### Youtube_Playlist_Downloader local -t *playlist_file* -a/--add *playlist:playlist_id/video:video_id*
Add videos or playlists to a local playlist  

E.g.  
```
    Youtube_Playlist_Downloader local -t cool_playlist.json -a playlist:PLH1TkRvPd30MSh23wIZTar8MUJ1w-XPB2
    Youtube_Playlist_Downloader local -t cool_playlist.json -a vidoe:67_S2YaoYC8
```

---

### Youtube_Playlist_Downloader local -t *playlist_file* -r/--remove *playlist:playlist_id/video:video_id*
Remove videos or playlists from a local playlist  

E.g.  
```
    Youtube_Playlist_Downloader local -t cool_playlist.json -r playlist:PLH1TkRvPd30MSh23wIZTar8MUJ1w-XPB2
    Youtube_Playlist_Downloader local -t cool_playlist.json -r vidoe:67_S2YaoYC8
```

---

### Youtube_Playlist_Downloader local -t *playlist_file* -d/--download
This option downloads the playlist and requires -t to be used  

E.g.  
```
    Youtube_Playlist_Downloader local -t cool_playlist.json -d
```
Would download cool_playlist.

---

### Youtube_Playlist_Downloader local -l/--list
Lists Contents of a local playlist

E.g.  
```
    Youtube_Playlist_Downloader local -t cool_playlist.json -l
```

---

### Youtube_Playlist_Downloader -o/--output *output directory* -n/--name *custom name* local -e/--edit
Edits the properties of a local playlist like the output directory and custom name.

E.g.  
```
    Youtube_Playlist_Downloader -o/--output LifeIsGreat -n/--name BestPlaylist-{VideoTitle} local -t playlist.json -e
    Youtube_Playlist_Downloader -n/--name BestPlaylist-{VideoTitle} local -e
    Youtube_Playlist_Downloader -o/--output LifeIsGreat local -e

```


---

## Global option
These apply to both remote and local playlits

---
### Youtube_Playlist_Downloader -o/--output *output directory*
Output directory for playlist. Can be absolute. It's case sensitive!  

E.g.
```
Youtube_Playlist_Downloader -o "real playlist" remote -f real_playlist
```
Will output all downloaded playlist's directories into *current_directory*/real playlist

This also applies to local --create

E.g.  
```
    Youtube_Playlist_Downloader -o "real playlist" local -c PLH1TkRvPd30MSh23wIZTar8MUJ1w-XPB2
```
This will make a file with the local playlist in that directory with the playlists title,  
if later downloaded from that file the playlist will be downloaded into that directory.  

---

## Youtube_Playlist_Downloader -n/-name *custom name*
You can put in video/playlist specific things in the name for e.g.  
"{PlaylistTitle} - {VideoTitle}" will be converted to "CoolPlaylist - Really cool song.opus".  
Variables that can be accesed by putting them in {}  
- VideoTitle
- PlaylistTitle
- CurrentDate
- ReleaseDate
- Author
- VideoID  

You can still include '{' and '}' in the file name by typing them twice to breakout of them!  

---

## Youtube_Playlist_Downloader -c/-cookies *file_with_cookies*
Obtain the file with your cookies by following the guide at  
https://github.com/yt-dlp/yt-dlp/wiki/Extractors#exporting-youtube-cookies  
I recommend using the private window method.  

---
