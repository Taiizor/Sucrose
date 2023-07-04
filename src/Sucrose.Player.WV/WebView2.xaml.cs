using Microsoft.Web.WebView2.Core;
using System.IO;
using System.Windows;
using SPSEH = Sucrose.Player.Shared.Event.Handler;
using SPWVEH = Sucrose.Player.WV.Event.Handler;
using SPWVMI = Sucrose.Player.WV.Manage.Internal;

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
                string tempPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "WebView2\\Cache");

                Task<CoreWebView2Environment> cwv2Environment = CoreWebView2Environment.CreateAsync(null, tempPath, new CoreWebView2EnvironmentOptions()
                {
                    AdditionalBrowserArguments = "--enable-media-stream --enable-accelerated-video-decode --allow-running-insecure-content --use-fake-ui-for-media-stream --enable-speech-input --enable-usermedia-screen-capture --debug-plugin-loading --allow-outdated-plugins --always-authorize-plugins --enable-npapi"
                }
                );

                SPWVMI.EdgePlayer.EnsureCoreWebView2Async(cwv2Environment.Result);

                SPSEH.WindowLoaded(this);
            };
        }
    }
}