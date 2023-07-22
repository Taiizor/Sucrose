using System.Windows;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHP = Sucrose.Engine.Shared.Helper.Properties;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SSEWVEW = Sucrose.Shared.Engine.WebView.Event.Web;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.View
{
    /// <summary>
    /// Interaction logic for Web.xaml
    /// </summary>
    public sealed partial class Web : Window
    {
        public Web(string Web)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SESEH.ContentRendered(this);

            Content = SSEWVMI.WebEngine;

            SSEWVMI.Web = Web;

            if (SESMI.Properties.State)
            {
                SESMI.PropertiesTimer.Tick += (s, e) => SESHP.ExecuteTask(SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync);
                SESMI.PropertiesTimer.Interval = TimeSpan.FromMilliseconds(SESMI.Properties.TriggerTime);
                SESMI.PropertiesTimer.Start();
            }

            SSEWVMI.WebEngine.CoreWebView2InitializationCompleted += SSEWVEW.WebEngineInitializationCompleted;

            Closing += (s, e) => SSEWVMI.WebEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }
    }
}