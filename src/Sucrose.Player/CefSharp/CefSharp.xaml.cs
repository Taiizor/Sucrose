using Sucrose.Player.Event;
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

            Content = Internal.CefPlayer;

            Internal.CefPlayer.Address = @"http://www.bokowsky.net/de/knowledge-base/video/videos/big_buck_bunny_240p.ogg"; //.webm - .mp4 - .ogg

            Internal.CefPlayer.BrowserSettings = Internal.CefSettings;

            Internal.CefPlayer.FrameLoadEnd += Handler.CefPlayerFrameLoadEnd;
            Internal.CefPlayer.Loaded += Handler.CefPlayerLoaded;

            Closing += (s, e) => Internal.EdgePlayer.Dispose();
            Loaded += (s, e) => Handler.WindowLoaded(this);
        }
    }
}