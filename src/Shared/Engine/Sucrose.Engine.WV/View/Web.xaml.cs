using System.Windows;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHP = Sucrose.Engine.Shared.Helper.Properties;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SEWVEW = Sucrose.Engine.WV.Event.Web;
using SEWVMI = Sucrose.Engine.WV.Manage.Internal;

namespace Sucrose.Engine.WV.View
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

            Content = SEWVMI.WebEngine;

            SEWVMI.Web = Web;

            if (SESMI.Properties.State)
            {
                SESMI.PropertiesTimer.Tick += (s, e) => SESHP.ExecuteTask(SEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync);
                SESMI.PropertiesTimer.Interval = TimeSpan.FromMilliseconds(SESMI.Properties.TriggerTime);
                SESMI.PropertiesTimer.Start();
            }

            SEWVMI.WebEngine.CoreWebView2InitializationCompleted += SEWVEW.WebEngineInitializationCompleted;

            Closing += (s, e) => SEWVMI.WebEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }
    }
}