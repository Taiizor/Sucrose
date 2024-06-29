using Microsoft.Win32;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSECSEU = Sucrose.Shared.Engine.CefSharp.Event.Url;
using SSECSHCCM = Sucrose.Shared.Engine.CefSharp.Handler.CustomContextMenu;
using SSECSHU = Sucrose.Shared.Engine.CefSharp.Helper.Url;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEHV = Sucrose.Shared.Engine.Helper.Volume;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.View
{
    /// <summary>
    /// Interaction logic for Url.xaml
    /// </summary>
    public sealed partial class Url : Window, IDisposable
    {
        public Url(string Url)
        {
            InitializeComponent();

            SystemEvents.DisplaySettingsChanged += (s, e) => SSEEH.DisplaySettingsChanged(this);

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            SSECSMI.CefEngine.MenuHandler = new SSECSHCCM();

            Content = SSECSMI.CefEngine;

            SSECSMI.Url = Url;

            SSECSMI.CefEngine.BrowserSettings = SSECSMI.CefSettings;

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            SSECSMI.CefEngine.IsBrowserInitializedChanged += SSECSEU.CefEngineInitializedChanged;
            SSECSMI.CefEngine.FrameLoadEnd += SSECSEU.CefEngineFrameLoadEnd;
            SSECSMI.CefEngine.Initialized += SSECSEU.CefEngineInitialized;
            SSECSMI.CefEngine.Loaded += SSECSEU.CefEngineLoaded;

            Closing += (s, e) => SSECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);

            SSEHV.Start();
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            Dispose();

            SSEHR.Control();

            SSECSHU.SetVolume(SSEHD.GetVolume());

            if (SMMM.PausePerformance)
            {
                SSECSHU.Pause();

                SSEMI.PausePerformance = true;
            }
            else if (SSEMI.PausePerformance)
            {
                SSECSHU.Play();

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