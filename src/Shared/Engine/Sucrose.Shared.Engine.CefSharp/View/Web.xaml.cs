using CefSharp;
using System.Windows;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHP = Sucrose.Engine.Shared.Helper.Properties;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SSECSEW = Sucrose.Shared.Engine.CefSharp.Event.Web;
using SSECSHCCM = Sucrose.Shared.Engine.CefSharp.Handler.CustomContextMenu;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.View
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

            SSECSMI.CefEngine.MenuHandler = new SSECSHCCM();

            Content = SSECSMI.CefEngine;

            SSECSMI.Web = Web;

            SSECSMI.CefEngine.BrowserSettings = SSECSMI.CefSettings;

            if (SESMI.Properties.State)
            {
                SESMI.PropertiesTimer.Tick += (s, e) => SESHP.ExecuteNormal(SSECSMI.CefEngine.ExecuteScriptAsync);
                SESMI.PropertiesTimer.Interval = TimeSpan.FromMilliseconds(SESMI.Properties.TriggerTime);
                SESMI.PropertiesTimer.Start();
            }

            SSECSMI.CefEngine.IsBrowserInitializedChanged += SSECSEW.CefEngineInitializedChanged;
            SSECSMI.CefEngine.Loaded += SSECSEW.CefEngineLoaded;

            Closing += (s, e) => SSECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }
    }
}