namespace Sucrose.Dependency.Enum
{
    internal enum EngineType
    {
        AppLive,
        WebViewLive,
        CefSharpLive,
        MediaElementLive
    }

    internal enum GifEngineType
    {
        Unknown = EngineType.MediaElementLive
    }

    internal enum UrlEngineType
    {
        WebView = EngineType.WebViewLive,
        CefSharp = EngineType.CefSharpLive
    }

    internal enum WebEngineType
    {
        WebView = EngineType.WebViewLive,
        CefSharp = EngineType.CefSharpLive
    }

    internal enum VideoEngineType
    {
        WebView = EngineType.WebViewLive,
        CefSharp = EngineType.CefSharpLive,
        MediaElement = EngineType.MediaElementLive
    }

    internal enum YouTubeEngineType
    {
        WebView = EngineType.WebViewLive,
        CefSharp = EngineType.CefSharpLive
    }

    internal enum ApplicationEngineType
    {
        App = EngineType.AppLive
    }
}