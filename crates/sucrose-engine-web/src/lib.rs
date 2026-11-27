//! The pure core of web wallpapers: the virtual origin every theme page is served from (D-14), the one function that
//! maps a request path to a file inside the theme folder or the generated-content folder (SEC-02), the `Range` math,
//! the MIME table, the profile allocator (D-33), the Chromium switches model (D-17), the legacy key table (D-13), the legacy-exact script strings
//! of the wallpaper JavaScript API (property and compatible calls, filedropdown lists, the in-page pause), the page state
//! and page calls both engines run (WAPI-01..04) and the
//! reply planner of the virtual origin both web engines stream from. No OS code and no unsafe: it builds and tests on
//! every OS, and each web engine puts a thin adapter on it (CEF's scheme handler, WebView2's WebResourceRequested).

pub mod args;
pub mod js;
pub mod keys;
pub mod mime;
pub mod moves;
pub mod origin;
pub mod page;
pub mod pages;
pub mod profile;
pub mod serve;
pub mod youtube;
