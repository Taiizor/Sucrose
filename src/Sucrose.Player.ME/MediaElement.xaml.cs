using System.Windows;
using SPMEEH = Sucrose.Player.ME.Event.Handler;
using SPMEHMEH = Sucrose.Player.ME.Helper.MediaElementHelper;
using SPMEMI = Sucrose.Player.ME.Manage.Internal;
using SPSEH = Sucrose.Player.Shared.Event.Handler;

namespace Sucrose.Player.ME
{
    /// <summary>
    /// Interaction logic for MediaElement.xaml
    /// </summary>
    public sealed partial class MediaElement : Window
    {
        public MediaElement()
        {
            InitializeComponent();

            Content = SPMEMI.MediaPlayer;

            SPMEMI.MediaPlayer.Source = new Uri(@"https://prod-streaming-video-msn-com.akamaized.net/ba33094e-efb2-480f-ad77-1a103af156d2/3b215666-f0b7-4538-9ea2-0637d5ed2ea3.mp4");

            SPMEMI.MediaPlayer.MediaOpened += SPMEEH.MediaPlayerOpened;
            SPMEMI.MediaPlayer.MediaFailed += SPMEEH.MediaPlayerFailed;
            SPMEMI.MediaPlayer.MediaEnded += SPMEEH.MediaPlayerEnded;

            Closing += (s, e) => SPMEMI.MediaPlayer.Close();
            Loaded += (s, e) => SPSEH.WindowLoaded(this);

            SPMEHMEH.SetVolume(100);

            SPMEMI.MediaPlayer.Play();
        }
    }
}