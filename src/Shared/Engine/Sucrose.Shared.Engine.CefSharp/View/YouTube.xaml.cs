using System.Windows;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SSECSEYT = Sucrose.Shared.Engine.CefSharp.Event.YouTube;
using SSECSHCCM = Sucrose.Shared.Engine.CefSharp.Handler.CustomContextMenu;
using SSECSHYT = Sucrose.Shared.Engine.CefSharp.Helper.YouTube;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.View
{
    /// <summary>
    /// Interaction logic for YouTube.xaml
    /// </summary>
    public sealed partial class YouTube : Window
    {
        public YouTube(string YouTube)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SESEH.ContentRendered(this);

            SSECSMI.CefEngine.MenuHandler = new SSECSHCCM();

            Content = SSECSMI.CefEngine;

            SSECSMI.YouTube = YouTube;

            SSECSMI.CefEngine.BrowserSettings = SSECSMI.CefSettings;

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            SSECSMI.CefEngine.IsBrowserInitializedChanged += SSECSEYT.CefEngineInitializedChanged;
            SSECSMI.CefEngine.FrameLoadEnd += SSECSEYT.CefEngineFrameLoadEnd;
            SSECSMI.CefEngine.Loaded += SSECSEYT.CefEngineLoaded;

            Closing += (s, e) => SSECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            if (SESMI.Initialized)
            {
                SSECSHYT.First();

                SSECSHYT.SetLoop(SESHD.GetLoop());

                SSECSHYT.SetVolume(SESHD.GetVolume());

                SSECSHYT.SetShuffle(SESHD.GetShuffle());
            }
        }
    }
}