//! The Referer rule of the YouTube hosts: a missing Referer is filled with the virtual origin for the player's own hosts, and
//! nothing else is ever touched.

use sucrose_engine_web::{
    origin::ORIGIN,
    youtube::{YOUTUBE_HOSTS, referer_for, request_filters},
};

#[test]
fn the_youtube_hosts_are_the_player_and_its_privacy_enhanced_twin() {
    assert_eq!(YOUTUBE_HOSTS, ["youtube.com", "youtube-nocookie.com"]);
}

#[test]
fn a_missing_referer_is_the_virtual_origin_for_a_youtube_request() {
    for url in [
        "https://www.youtube.com/embed/x",
        "https://www.youtube.com/iframe_api",
        "https://youtube.com/",
        "https://m.youtube.com/watch?v=x",
        "https://www.youtube-nocookie.com/embed/x?autoplay=1",
        "https://youtube-nocookie.com/embed/x",
        "https://WWW.YouTube.COM/embed/x",
        "https://www.youtube.com:443/embed/x",
        "https://www.youtube.com./embed/x",
    ] {
        assert_eq!(referer_for(url, None).as_deref(), Some(ORIGIN), "{url}");
    }
}

#[test]
fn a_referer_the_request_already_carries_is_never_replaced() {
    assert_eq!(
        referer_for("https://www.youtube.com/embed/x", Some("https://example.com/")),
        None
    );
    assert_eq!(referer_for("https://www.youtube.com/embed/x", Some(ORIGIN)), None);
    assert_eq!(referer_for("https://www.youtube.com/embed/x", Some("")), None);
}

#[test]
fn a_host_outside_the_youtube_list_is_never_touched() {
    for url in [
        "https://example.com/",
        "https://notyoutube.com/",
        "https://youtube.com.example.com/",
        "https://example.com/youtube.com/",
        "https://example.com/?next=https://www.youtube.com/",
        "https://youtube.com@example.com/",
        "https://example.com\\@www.youtube.com/",
        "https://www.youtube.com.evil/",
        "https://youtube-nocookie.com.evil/",
        "https://sucrose.wallpaper.engine/YouTubeContent.html",
        "http://www.youtube.com/embed/x",
        "ftp://www.youtube.com/",
        "www.youtube.com/embed/x",
        "",
    ] {
        assert_eq!(referer_for(url, None), None, "{url}");
    }
}

#[test]
fn the_request_filters_cover_the_hosts_and_their_subdomains_over_https_only() {
    assert_eq!(
        request_filters(),
        [
            "https://*.youtube.com/*",
            "https://youtube.com/*",
            "https://*.youtube-nocookie.com/*",
            "https://youtube-nocookie.com/*",
        ]
    );
}
