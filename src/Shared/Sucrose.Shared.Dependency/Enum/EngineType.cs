namespace Sucrose.Shared.Dependency.Enum
{
    internal enum EngineType
    {
        AuroraLive,
        NebulaLive,
        VexanaLive,
        WebViewLive,
        CefSharpLive
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
        Nebula = EngineType.NebulaLive,
        WebView = EngineType.WebViewLive,
        CefSharp = EngineType.CefSharpLive
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