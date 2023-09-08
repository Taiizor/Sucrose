using System.Windows;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVEU = Sucrose.Shared.Engine.WebView.Event.Url;
using SSEWVHU = Sucrose.Shared.Engine.WebView.Helper.Url;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.View
{
    /// <summary>
    /// Interaction logic for Url.xaml
    /// </summary>
    public sealed partial class Url : Window, IDisposable
    {
        public Url(string Url)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            Content = SSEWVMI.WebEngine;

            SSEWVMI.Url = Url;

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            SSEWVMI.WebEngine.CoreWebView2InitializationCompleted += SSEWVEU.WebEngineInitializationCompleted;

            Closing += (s, e) => SSEWVMI.WebEngine.Dispose();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            Dispose();

            SSEWVHU.SetVolume(SSEHD.GetVolume());
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}