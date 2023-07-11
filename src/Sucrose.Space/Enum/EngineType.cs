namespace Sucrose.Space.Enum
{
    public enum EngineType
    {
        AppLive,
        WebViewLive,
        CefSharpLive,
        MediaElementLive
    }

    public enum GifEngineType
    {
        Unknown = EngineType.MediaElementLive
    }

    public enum UrlEngineType
    {
        WebView = EngineType.WebViewLive,
        CefSharp = EngineType.CefSharpLive
    }

    public enum WebEngineType
    {
        WebView = EngineType.WebViewLive,
        CefSharp = EngineType.CefSharpLive
    }

    public enum VideoEngineType
    {
        WebView = EngineType.WebViewLive,
        CefSharp = EngineType.CefSharpLive,
        MediaElement = EngineType.MediaElementLive
    }

    public enum YouTubeEngineType
    {
        WebView = EngineType.WebViewLive,
        CefSharp = EngineType.CefSharpLive
    }

    public enum ApplicationEngineType
    {
        App = EngineType.AppLive
    }
}