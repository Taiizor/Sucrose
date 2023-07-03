using Sucrose.Player.Event;
using Sucrose.Player.Manage;
using System.Windows;

namespace Sucrose.Player.WebView2
{
    /// <summary>
    /// Interaction logic for WebView2.xaml
    /// </summary>
    public sealed partial class WebView2 : Window
    {
        public WebView2()
        {
            InitializeComponent();

            Content = Internal.EdgePlayer;

            Internal.EdgePlayer.Source = new Uri(@"https://prod-streaming-video-msn-com.akamaized.net/ba33094e-efb2-480f-ad77-1a103af156d2/3b215666-f0b7-4538-9ea2-0637d5ed2ea3.mp4");

            Internal.EdgePlayer.CoreWebView2InitializationCompleted += Handler.EdgePlayerInitializationCompleted;

            Closing += (s, e) => Internal.EdgePlayer.Dispose();
            Loaded += (s, e) => Handler.WindowLoaded(this);
        }
    }
}