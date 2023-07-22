using CefSharp;
using System.Windows;
using SSECSEW = Sucrose.Shared.Engine.CefSharp.Event.Web;
using SSECSHCCM = Sucrose.Shared.Engine.CefSharp.Handler.CustomContextMenu;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHC = Sucrose.Shared.Engine.Helper.Compatible;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

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

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            SSECSMI.CefEngine.MenuHandler = new SSECSHCCM();

            Content = SSECSMI.CefEngine;

            SSECSMI.Web = Web;

            SSECSMI.CefEngine.BrowserSettings = SSECSMI.CefSettings;

            if (SSEMI.Compatible.State)
            {
                SSEMI.CompatibleTimer.Tick += (s, e) => SSEHC.ExecuteNormal(SSECSMI.CefEngine.ExecuteScriptAsync);
                SSEMI.CompatibleTimer.Interval = TimeSpan.FromMilliseconds(SSEMI.Compatible.TriggerTime);
                SSEMI.CompatibleTimer.Start();
            }

            SSECSMI.CefEngine.IsBrowserInitializedChanged += SSECSEW.CefEngineInitializedChanged;
            SSECSMI.CefEngine.FrameLoadEnd += SSECSEW.CefEngineFrameLoadEnd;
            SSECSMI.CefEngine.Loaded += SSECSEW.CefEngineLoaded;

            Closing += (s, e) => SSECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);
        }
    }
}