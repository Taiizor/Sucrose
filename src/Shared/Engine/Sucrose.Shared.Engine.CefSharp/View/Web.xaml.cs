using Microsoft.Win32;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSECSEW = Sucrose.Shared.Engine.CefSharp.Event.Web;
using SSECSHCCM = Sucrose.Shared.Engine.CefSharp.Handler.CustomContextMenu;
using SSECSHW = Sucrose.Shared.Engine.CefSharp.Helper.Web;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEHV = Sucrose.Shared.Engine.Helper.Volume;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.View
{
    /// <summary>
    /// Interaction logic for Web.xaml
    /// </summary>
    public sealed partial class Web : Window, IDisposable
    {
        public Web(string Web)
        {
            InitializeComponent();

            SystemEvents.DisplaySettingsChanged += (s, e) => SSEEH.DisplaySettingsChanged(this);

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            SSECSMI.CefEngine.MenuHandler = new SSECSHCCM();

            Content = SSECSMI.CefEngine;

            SSECSMI.Web = Web;

            SSECSMI.CefEngine.BrowserSettings = SSECSMI.CefSettings;

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            SSECSMI.CefEngine.IsBrowserInitializedChanged += SSECSEW.CefEngineInitializedChanged;
            SSECSMI.CefEngine.FrameLoadEnd += SSECSEW.CefEngineFrameLoadEnd;
            SSECSMI.CefEngine.Loaded += SSECSEW.CefEngineLoaded;

            Closing += (s, e) => SSECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);

            SSEHV.Start();
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            Dispose();

            SSEHR.Control();

            SSECSHW.SetVolume(SSEHD.GetVolume());

            if (SMMM.PausePerformance)
            {
                SSECSHW.Pause();

                SSEMI.PausePerformance = true;
            }
            else if (SSEMI.PausePerformance)
            {
                SSECSHW.Play();

                SSEMI.PausePerformance = false;
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}