using Microsoft.Web.WebView2.Core;
using System.IO;
using System.Windows;
using SPSEH = Sucrose.Player.Shared.Event.Handler;
using SPWVEH = Sucrose.Player.WV.Event.Handler;
using SPWVMI = Sucrose.Player.WV.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Player.WV
{
    /// <summary>
    /// Interaction logic for WebView2.xaml
    /// </summary>
    public sealed partial class WebView2 : Window
    {
        public WebView2()
        {
            InitializeComponent();

            Content = SPWVMI.EdgePlayer;

            SPWVMI.EdgePlayer.Source = new Uri(@"https://prod-streaming-video-msn-com.akamaized.net/ba33094e-efb2-480f-ad77-1a103af156d2/3b215666-f0b7-4538-9ea2-0637d5ed2ea3.mp4");

            SPWVMI.EdgePlayer.CoreWebView2InitializationCompleted += SPWVEH.EdgePlayerInitializationCompleted;

            Closing += (s, e) => SPWVMI.EdgePlayer.Dispose();
            Loaded += (s, e) =>
            {
                CoreWebView2EnvironmentOptions Options = new()
                {
                    AdditionalBrowserArguments = "--enable-media-stream --enable-accelerated-video-decode --allow-running-insecure-content --use-fake-ui-for-media-stream --enable-speech-input --enable-usermedia-screen-capture --debug-plugin-loading --allow-outdated-plugins --always-authorize-plugins --enable-npapi"
                };

                Task<CoreWebView2Environment> Environment = CoreWebView2Environment.CreateAsync(null, Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.WebView2), Options);

                SPWVMI.EdgePlayer.EnsureCoreWebView2Async(Environment.Result);

                SPSEH.WindowLoaded(this);
            };
        }
    }
}