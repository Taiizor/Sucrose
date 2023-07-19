using CefSharp;
using System.Windows;
using SDEST = Sucrose.Dependency.Enum.StretchType;
using SECSEW = Sucrose.Engine.CS.Event.Web;
using SECSHCCM = Sucrose.Engine.CS.Handler.CustomContextMenu;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;

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

            SESMI.PropertiesTimer.Interval = new TimeSpan(0, 0, SESMI.Properties.TriggerTime);
            SESMI.PropertiesTimer.Tick += new EventHandler(PropertiesTimer_Tick);
            SESMI.PropertiesTimer.Start();

            SECSMI.CefEngine.Loaded += SECSEW.CefEngineLoaded;

            Closing += (s, e) => SECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }

        private void PropertiesTimer_Tick(object sender, EventArgs e)
        {
            if (!string.IsNullOrEmpty(SESMI.Properties.StretchMode))
            {
                SECSMI.CefEngine.ExecuteScriptAsync(string.Format(SESMI.Properties.StretchMode, SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SDEST.Fill)));
            }
        }
    }
}