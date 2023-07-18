using System.Windows;
using SDEST = Sucrose.Dependency.Enum.StretchType;
using SECSEV = Sucrose.Engine.CS.Event.Video;
using SECSHCCM = Sucrose.Engine.CS.Handler.CustomContextMenu;
using SECSHV = Sucrose.Engine.CS.Helper.Video;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Engine.CS.View
{
    /// <summary>
    /// Interaction logic for Video.xaml
    /// </summary>
    public sealed partial class Video : Window
    {
        public Video(string Video)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SESEH.ContentRendered(this);

            SECSMI.CefEngine.MenuHandler = new SECSHCCM();

            Content = SECSMI.CefEngine;

            SECSMI.CefEngine.Address = SESHS.GetSource(Video).ToString();

            SECSMI.CefEngine.BrowserSettings = SECSMI.CefSettings;

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            SECSMI.CefEngine.FrameLoadEnd += SECSEV.CefEngineFrameLoadEnd;
            SECSMI.CefEngine.Loaded += SECSEV.CefEngineLoaded;

            Closing += (s, e) => SECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            SECSHV.SetLoop(SMMI.EngineSettingManager.GetSetting(SMC.Loop, true));

            SECSHV.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SDEST Stretch = SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SDEST.Fill);

            if ((int)Stretch < Enum.GetValues(typeof(SDEST)).Length)
            {
                SECSHV.SetStretch(Stretch);
            }
        }
    }
}