//! The pure core of web wallpapers: the virtual origin every theme page is served from (D-14), the one function that
//! maps a request path to a file inside the theme folder or the generated-content folder (SEC-02), the `Range` math
//! and the MIME table. No OS code and no unsafe: it builds and tests on every OS, and each web engine puts a thin
//! adapter on it (CEF's scheme handler now, WebView2's WebResourceRequested later).

pub mod mime;
pub mod origin;
