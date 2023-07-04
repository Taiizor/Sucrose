using CefSharp;
using CefSharp.Wpf;
using Skylark.Enum;
using Skylark.Wing;
using System.IO;
using System.Windows;

namespace Sucrose.Test
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        public App()
        {
            CefSettings settings = new()
            {
                //By default CefSharp will use an in-memory cache, you need to specify a Cache Folder to persist data
                CachePath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "CefSharp\\Cache"),
                IgnoreCertificateErrors = true
            };

            settings.CefCommandLineArgs.Add("autoplay-policy", "no-user-gesture-required");

            settings.CefCommandLineArgs.Add("enable-media-stream", "1");
            settings.CefCommandLineArgs.Add("enable-accelerated-video-decode", "1");

            settings.CefCommandLineArgs.Add("allow-running-insecure-content", "1");
            settings.CefCommandLineArgs.Add("use-fake-ui-for-media-stream", "1");
            settings.CefCommandLineArgs.Add("enable-speech-input", "1");
            settings.CefCommandLineArgs.Add("enable-usermedia-screen-capture", "1");
            settings.CefCommandLineArgs.Add("debug-plugin-loading", "1");
            settings.CefCommandLineArgs.Add("allow-outdated-plugins", "1");
            settings.CefCommandLineArgs.Add("always-authorize-plugins", "1");
            settings.CefCommandLineArgs.Add("enable-npapi", "1");

            Cef.Initialize(settings, performDependencyCheck: false, browserProcessHandler: null);
















            Sucrose.Player.WV.WebView2 Player = new();
            Player.Show();

            Engine.WallpaperWindow(Player, 0, ScreenType.DisplayBound);
        }
    }
}