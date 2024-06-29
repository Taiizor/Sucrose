using Microsoft.Win32;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEHV = Sucrose.Shared.Engine.Helper.Volume;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVEW = Sucrose.Shared.Engine.WebView.Event.Web;
using SSEWVHW = Sucrose.Shared.Engine.WebView.Helper.Web;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.View
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

            Content = SSEWVMI.WebEngine;

            SSEWVMI.Web = Web;

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            SSEWVMI.WebEngine.CoreWebView2InitializationCompleted += SSEWVEW.WebEngineInitializationCompleted;

            Closing += (s, e) => SSEWVMI.WebEngine.Dispose();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);

            SSEHV.Start();
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            Dispose();

            SSEHR.Control();

            SSEWVHW.SetVolume(SSEHD.GetVolume());

            if (SMMM.PausePerformance)
            {
                SSEWVHW.Pause();

                SSEMI.PausePerformance = true;
            }
            else if (SSEMI.PausePerformance)
            {
                SSEWVHW.Play();

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