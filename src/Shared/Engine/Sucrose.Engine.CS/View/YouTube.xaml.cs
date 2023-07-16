using System.Windows;
using System.Windows.Threading;
using SECSEYT = Sucrose.Engine.CS.Event.YouTube;
using SECSHCCM = Sucrose.Engine.CS.Handler.CustomContextMenu;
using SECSHYT = Sucrose.Engine.CS.Helper.YouTube;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Engine.CS.View
{
    /// <summary>
    /// Interaction logic for YouTube.xaml
    /// </summary>
    public sealed partial class YouTube : Window
    {
        private readonly DispatcherTimer Timer = new();

        public YouTube(string YouTube)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SESEH.ContentRendered(this);

            SECSMI.CefEngine.MenuHandler = new SECSHCCM();

            Content = SECSMI.CefEngine;

            SECSMI.YouTube = YouTube;

            SECSMI.CefEngine.BrowserSettings = SECSMI.CefSettings;

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SECSMI.CefEngine.FrameLoadEnd += SECSEYT.CefEngineFrameLoadEnd;
            SECSMI.CefEngine.Loaded += SECSEYT.CefEngineLoaded;

            Closing += (s, e) => SECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            SECSHYT.SetLoop(SMMI.EngineSettingManager.GetSetting(SMC.Loop, true));

            SECSHYT.SetShuffle(SMMI.EngineSettingManager.GetSetting(SMC.Shuffle, true));

            SECSHYT.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));
        }
    }
}