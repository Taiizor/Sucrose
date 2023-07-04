using System.Windows;
using SPCSEH = Sucrose.Player.CS.Event.Handler;
using SPCSMI = Sucrose.Player.CS.Manage.Internal;
using SPSEH = Sucrose.Player.Shared.Event.Handler;

namespace Sucrose.Player.CS
{
    /// <summary>
    /// Interaction logic for CefSharp.xaml
    /// </summary>
    public sealed partial class CefSharp : Window
    {
        public CefSharp()
        {
            InitializeComponent();

            Content = SPCSMI.CefPlayer;

            SPCSMI.CefPlayer.Address = @"http://www.bokowsky.net/de/knowledge-base/video/videos/big_buck_bunny_240p.ogg"; //.webm - .mp4 - .ogg

            SPCSMI.CefPlayer.BrowserSettings = SPCSMI.CefSettings;

            SPCSMI.CefPlayer.FrameLoadEnd += SPCSEH.CefPlayerFrameLoadEnd;
            SPCSMI.CefPlayer.Loaded += SPCSEH.CefPlayerLoaded;

            Closing += (s, e) => SPCSMI.CefPlayer.Dispose();
            Loaded += (s, e) => SPSEH.WindowLoaded(this);
        }
    }
}