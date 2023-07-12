namespace Sucrose.Dependency.Enum
{
    internal enum EngineType
    {
        AuroraLive,
        VexanaLive,
        WebViewLive,
        CefSharpLive,
        MediaElementLive //FreyaLive, AureliaLive, NovaraLive, 
    }

    internal enum GifEngineType
    {
        Vexana = EngineType.VexanaLive
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
        Aurora = EngineType.AuroraLive
    }
}