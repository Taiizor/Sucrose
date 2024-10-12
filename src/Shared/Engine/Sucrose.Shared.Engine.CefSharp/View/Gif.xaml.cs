using Microsoft.Win32;
using System.Windows;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SSECSEG = Sucrose.Shared.Engine.CefSharp.Event.Gif;
using SSECSHCCM = Sucrose.Shared.Engine.CefSharp.Handler.CustomContextMenu;
using SSECSHG = Sucrose.Shared.Engine.CefSharp.Helper.Gif;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.View
{
    /// <summary>
    /// Interaction logic for Gif.xaml
    /// </summary>
    public sealed partial class Gif : Window, IDisposable
    {
        public Gif(string Gif)
        {
            InitializeComponent();

            SystemEvents.DisplaySettingsChanged += (s, e) => SSEEH.DisplaySettingsChanged(this);

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            SSECSMI.CefEngine.MenuHandler = new SSECSHCCM();

            Content = SSECSMI.CefEngine;

            SSECSMI.Gif = Gif;

            SSECSMI.CefEngine.BrowserSettings = SSECSMI.CefSettings;

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            SSECSMI.CefEngine.IsBrowserInitializedChanged += SSECSEG.CefEngineInitializedChanged;
            SSECSMI.CefEngine.FrameLoadEnd += SSECSEG.CefEngineFrameLoadEnd;
            SSECSMI.CefEngine.Loaded += SSECSEG.CefEngineLoaded;

            Closing += (s, e) => SSECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            if (SSEMI.Initialized)
            {
                Dispose();

                SSEHR.Control();

                SSECSHG.SetLoop(SSEHD.GetLoop());

                SSECSHG.SetStretch(SSEHD.GetStretch());

                if (SMMB.PausePerformance)
                {
                    SSECSHG.Pause();

                    SSEMI.PausePerformance = true;
                }
                else if (SSEMI.PausePerformance)
                {
                    SSECSHG.Play();

                    SSEMI.PausePerformance = false;
                }
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}