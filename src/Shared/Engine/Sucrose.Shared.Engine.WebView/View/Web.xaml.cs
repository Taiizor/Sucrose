using System.Windows;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHP = Sucrose.Shared.Engine.Helper.Properties;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
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

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            Content = SSEWVMI.WebEngine;

            SSEWVMI.Web = Web;

            if (SSEMI.Properties.State)
            {
                SSEMI.PropertiesTimer.Tick += (s, e) => SSEHP.ExecuteTask(SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync);
                SSEMI.PropertiesTimer.Interval = TimeSpan.FromMilliseconds(SSEMI.Properties.TriggerTime);
                SSEMI.PropertiesTimer.Start();
            }

            SSEWVMI.WebEngine.CoreWebView2InitializationCompleted += SSEWVEW.WebEngineInitializationCompleted;

            Closing += (s, e) => SSEWVMI.WebEngine.Dispose();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);
        }
    }
}