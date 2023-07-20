using CefSharp;
using System.Windows;
using SECSEW = Sucrose.Engine.CS.Event.Web;
using SECSHCCM = Sucrose.Engine.CS.Handler.CustomContextMenu;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHP = Sucrose.Engine.Shared.Helper.Properties;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;

namespace Sucrose.Engine.CS.View
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

            SECSMI.CefEngine.MenuHandler = new SECSHCCM();

            Content = SECSMI.CefEngine;

            SECSMI.Web = Web;

            SECSMI.CefEngine.BrowserSettings = SECSMI.CefSettings;

            if (SECSMI.CefEngine.IsBrowserInitialized && SESMI.Properties.State)
            {
                SESMI.PropertiesTimer.Tick += (s, e) => SESHP.ExecuteNormal(SECSMI.CefEngine.ExecuteScriptAsync);
                SESMI.PropertiesTimer.Interval = TimeSpan.FromMilliseconds(SESMI.Properties.TriggerTime);
                SESMI.PropertiesTimer.Start();
            }

            SECSMI.CefEngine.Loaded += SECSEW.CefEngineLoaded;

            Closing += (s, e) => SECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }
    }
}