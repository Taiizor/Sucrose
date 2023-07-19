using System.Windows;
using SECSEV = Sucrose.Engine.CS.Event.Video;
using SECSHCCM = Sucrose.Engine.CS.Handler.CustomContextMenu;
using SECSHV = Sucrose.Engine.CS.Helper.Video;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;

namespace Sucrose.Engine.CS.View
{
    /// <summary>
    /// Interaction logic for Video.xaml
    /// </summary>
    public sealed partial class Video : Window
    {
        public Video(string Video)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SESEH.ContentRendered(this);

            SECSMI.CefEngine.MenuHandler = new SECSHCCM();

            Content = SECSMI.CefEngine;

            SECSMI.CefEngine.Address = SESHS.GetSource(Video).ToString();

            SECSMI.CefEngine.BrowserSettings = SECSMI.CefSettings;

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            SECSMI.CefEngine.FrameLoadEnd += SECSEV.CefEngineFrameLoadEnd;
            SECSMI.CefEngine.Loaded += SECSEV.CefEngineLoaded;

            Closing += (s, e) => SECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            SECSHV.SetLoop(SESHD.GetLoop());

            SECSHV.SetVolume(SESHD.GetVolume());

            SECSHV.SetStretch(SESHD.GetStretch());
        }
    }
}