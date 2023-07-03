using Sucrose.Player.Event;
using Sucrose.Player.Helper;
using Sucrose.Player.Manage;
using System.Windows;

namespace Sucrose.Player.CefSharp
{
    /// <summary>
    /// Interaction logic for CefSharp.xaml
    /// </summary>
    public sealed partial class CefSharp : Window
    {
        public CefSharp()
        {
            InitializeComponent();

            Content = Internal.MediaPlayer;

            Internal.MediaPlayer.Source = new Uri(@"https://prod-streaming-video-msn-com.akamaized.net/ba33094e-efb2-480f-ad77-1a103af156d2/3b215666-f0b7-4538-9ea2-0637d5ed2ea3.mp4");

            Internal.MediaPlayer.MediaOpened += Handler.MediaPlayerOpened;
            Internal.MediaPlayer.MediaFailed += Handler.MediaPlayerFailed;
            Internal.MediaPlayer.MediaEnded += Handler.MediaPlayerEnded;

            Closing += (s, e) => Internal.MediaPlayer.Close();
            Loaded += (s, e) => Handler.WindowLoaded(this);

            MediaElementHelper.SetVolume(100);

            Internal.MediaPlayer.Play();
        }
    }
}