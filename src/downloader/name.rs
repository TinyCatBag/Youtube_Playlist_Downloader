use std::usize;
use serde::{Deserialize, Serialize};

use crate::downloader::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NamePart {
    //Titles
    VideoTitle,
    PlaylistTitle,
    //Dates
    CurrentDate,
    ReleaseDate,
    //ID's
    VideoID,
    PlaylistID,
    //Misc
    Author,
    String(String),     //Can't be bothered to deal with references
}

// Make it impossible to have an empty name >*-*<
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NameWhole{
    first: NamePart,
    rest: Vec<NamePart>,
}

impl Default for NameWhole {
    fn default() -> Self {
        Self {
            first: NamePart::PlaylistTitle,
            rest: vec![NamePart::String(" - ".to_string()), NamePart::VideoTitle]
        }
    }
}

impl NameWhole {
    fn new(first: NamePart, rest: Vec<NamePart>) -> Self{
        NameWhole {
            first,
            rest
        }
    }
    fn new_from_first(first: NamePart) -> Self {
        Self {
            first,
            rest: vec![],
        }
    }
    pub fn from_string(str: &Option<&str>) -> Self {
        match str {
            Some(name) if name.is_empty() => {
                warn!("Name is empty! using the default.");
                NameWhole::default()
           }
            Some(name) if name.find("{").is_none() && name.find("}").is_none() => {
                warn!("ALL videos will have the same name!");
                NameWhole::new_from_first(NamePart::String(name.to_string()))
            }
            Some(name) if name.matches("{").count() - name.matches("{{").count()*2 > name.matches("}").count() - name.matches("}}").count()*2 => {
                panic!(r#"Unclosed '{{' you can breakout from it by typing it twice in a row '{{{{'"#);
            }
            Some(name) if name.matches("{").count() - name.matches("{{").count()*2 < name.matches("}").count() - name.matches("}}").count()*2 => {
                panic!(r#"Unopened '}}' you can breakout from it by typing it twice in a row '}}}}'"#);
            }
            Some(name) if name.find("{").is_some() && name.find("}").is_some() => {
                let mut first = NamePart::String("".to_string());
                let mut rest: Vec<NamePart> = vec![];
                let mut inner_str = &name[..];
                trace!("inner_str {inner_str}");
                loop {                    //TODO: Should probably handle this error or smth
                    let start = match NamePart::find_next_start(inner_str) {
                        Some(start) => start,
                        None => {
                            trace!("Found no more {{");
                            if inner_str.is_empty() {
                                trace!("String was empty");
                                break;
                            }
                            if first == NamePart::String("".to_string()){
                                first = NamePart::str_to_part(&inner_str[..]);
                                trace!("First was empty and is now {:?}", first);
                            }
                            else {
                                trace!("Pushing into rest: {:?}", NamePart::str_to_part(&inner_str[..]));
                                rest.push(NamePart::str_to_part(&inner_str[..]));
                            }
                            break;
                        },
                    };
                    let end = match NamePart::find_next_end(&inner_str[start..]) {
                        Some(end) => end,
                        None => {
                            trace!("Found no more }}");
                            if inner_str.is_empty() {
                                trace!("String was empty");
                                break;
                            }
                            if first == NamePart::String("".to_string()){
                                first = NamePart::str_to_part(&inner_str[..]);
                                trace!("First was empty and is now {:?}", first);
                            }
                            else {
                                trace!("Pushing into rest: {:?}", NamePart::str_to_part(&inner_str[..]));
                                rest.push(NamePart::str_to_part(&inner_str[..]));
                            }
                            break;
                        },
                    };
                    trace!("Start: {start}, End: {end}");
                    if start != 1  {
                        trace!("Start isnt 1");
                        if first == NamePart::String("".to_string()){
                            first = NamePart::String(inner_str[..start-1].to_string());
                            trace!("First was empty and is now {:?}", first);
                        }
                        else{
                            trace!("Pushing into rest: {:?}", NamePart::String(inner_str[..start-1].to_string()));
                            rest.push(NamePart::String(inner_str[..start-1].to_string()));
                        }
                    }
                    if first == NamePart::String("".to_string()){
                        first = NamePart::str_to_part(&inner_str[start..end+start]);
                        trace!("First was empty and is now {:?}", first);
                    }
                     else{
                        trace!("Pushing into rest: {:?}", NamePart::str_to_part(&inner_str[start..end+start]));
                        rest.push(NamePart::str_to_part(&inner_str[start..end+start]));
                    }
                    inner_str = &inner_str[end+start+1..];
                    trace!("inner_str after loop: {inner_str}");
                    trace!("first after loop: {:?}", first);
                    trace!("rest after loop: {:?}", rest);
                }
                let foo = NameWhole::new(first, rest);
                debug!("{:?}", foo);
                foo
            }
            Some(_name) => {
                panic!("Non exhaustive checking please report this on my github with the full command, Thanks")
            }
            None => NameWhole::default()
        }
    }
    pub fn formatted_download_path(self: &Self, video: &Video, playlist: &Playlist) -> String {
        let mut download_name = String::new();
        download_name.push_str(Self::namepart_to_string(&self.first, video, playlist));
        for namepart in &self.rest {
            download_name.push_str(Self::namepart_to_string(namepart, video, playlist));
        }
        download_name.push_str(".opus");
        //TODO: we could do this faster just look below
        format!("{}/{}", playlist.path.display(), download_name.replace("{{", "{").replace("}}", "}"))
    }
    pub fn formatted_video_title(self: &Self, video: &Video, playlist: &Playlist) -> String {
        let mut download_name = String::new();
        download_name.push_str(Self::namepart_to_string(&self.first, video, playlist));
        for namepart in &self.rest {
            download_name.push_str(Self::namepart_to_string(namepart, video, playlist));
        }
        download_name
    }
    fn namepart_to_string<'a>(namepart: &'a NamePart, video: &'a Video, playlist: &'a Playlist) -> &'a str {
        match namepart {
            NamePart::VideoTitle => &video.title[..],
            NamePart::PlaylistTitle => &playlist.title[..],
            NamePart::ReleaseDate => todo!("Dates not done"),
            NamePart::CurrentDate => todo!("Dates not done"),
            NamePart::VideoID => &video.id[..],
            NamePart::PlaylistID => &playlist.id[..],
            NamePart::Author => &video.author[..],
            NamePart::String(string) => {
                string
                //TODO: this can be solved with Cow<String> but is it worth it?
                // string = &string.replace("{{", "{").replace("}}", "}");
                // return string;
            },
        }
    }
}

impl NamePart {
    fn find_next_start(outer_str: &str) -> Option<usize> {
        let mut inner_str = outer_str;
        let mut outer_start = 0;
        loop {
            trace!("Outer_start = {outer_start}");
            if let Some(inner_start) = inner_str.find("{") {
                trace!("Found {{ at {inner_start}");
                if inner_str.chars().nth(inner_start+1).unwrap_or(' ') == '{' {
                    trace!("Found {{{{ at {}", inner_start+1);
                    outer_start += inner_start+2;
                    inner_str = &outer_str[outer_start..];
                    trace!("New inner_str {}", inner_str);
                    continue;
                }
                else {
                    outer_start += inner_start+1;
                    trace!("Didnt finnd {{{{ breaking on {}", outer_start);
                    trace!("string from that point '{}'", &outer_str[outer_start..]);
                    break
                }
            }
            else {
                trace!("Found no next {{");
                return None
            }
        }
        Some(outer_start)
    }
    fn find_next_end(outer_str: &str) -> Option<usize> {
        let mut inner_str = outer_str;
        let mut outer_start = 0;
        loop {
            trace!("Outer_start = {outer_start}");
            if let Some(inner_start) = inner_str.find("}") {

                trace!("Found }} at {inner_start}");
                if inner_str.chars().nth(inner_start+1).unwrap_or(' ') == '}' {
                    trace!("Found }}}} at {}", inner_start+1);
                    outer_start += inner_start+2;
                    inner_str = &outer_str[outer_start..];
                    trace!("New inner_str {}", inner_str);
                    continue;
                }
                else {
                    outer_start += inner_start;
                    trace!("Didnt finnd }}}} breaking on {}", outer_start);
                    trace!("string from that point '{}'", &outer_str[outer_start..]);
                    break
                }
            }
            else {
                trace!("Found no next }}");
                return None
            }
        }
        Some(outer_start)
    }
    fn str_to_part(str: &str) -> Self {
        let inner_string = str.to_lowercase();
        match inner_string.as_ref() {
            "videotitle"    => Self::VideoTitle,
            "playlisttitle" => Self::PlaylistTitle,
            "currentdate"   => Self::CurrentDate,
            "releasedate"   => Self::ReleaseDate,
            "videoid"       => Self::VideoID,
            "playlistid"    => Self::PlaylistID,
            "author"        => Self::Author,
            _               => {
                debug!("String found: {str}");
                // warn!("No match for variable provided in {{{inner_string}}}, for a list of them do -h");
                Self::String(str.to_string())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_to_part_videotitle() {
        assert_eq!(NamePart::str_to_part("videotitle"), NamePart::VideoTitle);
    }
    #[test]
    fn string_to_part_playlisttitle() {
        assert_eq!(NamePart::str_to_part("playlisttitle"), NamePart::PlaylistTitle);
    }
    #[test]
    fn string_to_part_currentdate() {
        assert_eq!(NamePart::str_to_part("currentdate"), NamePart::CurrentDate);
    }
    #[test]
    fn string_to_part_releasedate() {
        assert_eq!(NamePart::str_to_part("releasedate"), NamePart::ReleaseDate);
    }
    #[test]
    fn string_to_part_author() {
        assert_eq!(NamePart::str_to_part("author"), NamePart::Author);
    }
    #[test]
    fn string_to_part_videoid() {
        assert_eq!(NamePart::str_to_part("videoid"), NamePart::VideoID);
    }
    #[test]
    fn string_to_part_string() {
        assert_eq!(NamePart::str_to_part("strIIIIIng"), NamePart::String("strIIIIIng".to_string()));
    }
    #[test]
    fn find_next_start() {
        assert_eq!(NamePart::find_next_start("Hello {{wow}} {{Wowzav2}} - {PlaylistTitle}"), Some(29));
    }
    #[test]
    fn find_next_end() {
        assert_eq!(NamePart::find_next_end("Hello {{wow}} {{Wowzav2}} - {PlaylistTitle}"), Some(42));
        let start = NamePart::find_next_start("Hello {{wow}} {{Wowzav2}} - {PlaylistTitle}").unwrap();
        let str = &"Hello {{wow}} {{Wowzav2}} - {PlaylistTitle}"[start..];
        assert_eq!(NamePart::find_next_end(str), Some(13));
    }
    #[test]
    fn find_value() {
        let str_original = "Hello {{wow}} {{Wowzav2}} - {PlaylistTitle}";
        let start = NamePart::find_next_start("Hello {{wow}} {{Wowzav2}} - {PlaylistTitle}").unwrap();
        let str = &"Hello {{wow}} {{Wowzav2}} - {PlaylistTitle}"[start..];
        let end = NamePart::find_next_end(str).unwrap();
        assert_eq!(&str_original[start..end+start], "PlaylistTitle");
        assert_eq!(NamePart::str_to_part(&str_original[start..end+start]), NamePart::PlaylistTitle)
    }
    #[test]
    fn find_value_whole() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).is_test(true).try_init();
        let str = "{PlaylistTitle} - {VideoTitle}";
        assert_eq!(NameWhole::from_string(&Some(&str)), NameWhole::new(NamePart::PlaylistTitle, vec![NamePart::String(" - ".to_string()), NamePart::VideoTitle]))
    }
    #[test]
    fn find_value_whole_v2() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).is_test(true).try_init();
        let str = "{PlaylistTitle} - {VideoTitle} - Whoo look at me hehe";
        assert_eq!(NameWhole::from_string(&Some(&str)), NameWhole::new(NamePart::PlaylistTitle, vec![
            NamePart::String(" - ".to_string()),
            NamePart::VideoTitle,
            NamePart::String(" - Whoo look at me hehe".to_string())
        ]))
    }
    #[test]
    fn find_value_whole_with_breakouts() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).is_test(true).try_init();
        let str = "{{PlaylistTitle}} - {VideoTitle} - {Author} - {{VideoID}} - Whoo look at me hehe";
        assert_eq!(NameWhole::from_string(&Some(&str)), NameWhole::new(NamePart::String("{{PlaylistTitle}} - ".to_string()), vec![
            NamePart::VideoTitle,
            NamePart::String(" - ".to_string()),
            NamePart::Author,
            NamePart::String(" - {{VideoID}} - Whoo look at me hehe".to_string())
        ]))
    }
}
