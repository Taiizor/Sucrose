//! The Referer rule of the YouTube hosts (D-14, research Findings 12). The generated YouTube page is a real document on the
//! virtual origin, so the browser sends that origin as the Referer of the player's requests by itself; both web engines
//! still keep a small hook as a safety net that fills a Referer a request lacks, and this is the one rule they share. It
//! fills only a missing Referer, only for the player's own hosts, and only with the virtual origin (T-04-75). No OS code
//! and no unsafe.

use crate::origin::ORIGIN;

/// The hosts whose requests may get a Referer filled in: the YouTube player and its privacy-enhanced twin. A subdomain of
/// one of them counts; a host that merely ends in the same letters does not.
pub const YOUTUBE_HOSTS: [&str; 2] = ["youtube.com", "youtube-nocookie.com"];

/// The host of an `https` address (for a log line or a filter decision), lower case and without its user name, port or a trailing dot; `None` for any other
/// scheme or an address without a host. A backslash ends the authority as a slash does (a browser reads it so), so
/// `https://example.com\@www.youtube.com/` is the host `example.com`.
pub fn https_host(url: &str) -> Option<String> {
    let rest = url.get(..8).filter(|scheme| scheme.eq_ignore_ascii_case("https://"))?;
    let rest = &url[rest.len()..];
    let authority = rest.split(['/', '\\', '?', '#']).next().unwrap_or_default();
    let host = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    let host = host.rsplit_once(':').map_or(host, |(name, port)| {
        if port.bytes().all(|byte| byte.is_ascii_digit()) {
            name
        } else {
            host
        }
    });
    let host = host.trim_end_matches('.');
    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

/// Whether `host` is one of [`YOUTUBE_HOSTS`] or a subdomain of one.
fn is_youtube_host(host: &str) -> bool {
    YOUTUBE_HOSTS
        .iter()
        .any(|known| host == *known || host.strip_suffix(known).is_some_and(|head| head.ends_with('.')))
}

/// The Referer to put on the request for `request_url`: the virtual origin when the request goes to a YouTube host over
/// https and has no Referer yet (`existing` is `None`), otherwise `None`. A Referer the request already carries is
/// never replaced, whatever it says.
#[must_use]
pub fn referer_for(request_url: &str, existing: Option<&str>) -> Option<String> {
    if existing.is_some() {
        return None;
    }
    https_host(request_url)
        .filter(|host| is_youtube_host(host))
        .map(|_| ORIGIN.to_owned())
}

/// The URL filters a web engine registers to see the requests [`referer_for`] applies to: each host of
/// [`YOUTUBE_HOSTS`] and its subdomains, https only (WebView2's `AddWebResourceRequestedFilter` matches the whole URL, and
/// `*.host` does not match the bare host).
#[must_use]
pub fn request_filters() -> Vec<String> {
    YOUTUBE_HOSTS
        .iter()
        .flat_map(|host| [format!("https://*.{host}/*"), format!("https://{host}/*")])
        .collect()
}
