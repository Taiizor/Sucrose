using System.Windows;
using System.Windows.Threading;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPCSEVH = Sucrose.Player.CS.Event.VideoHandler;
using SPCSHVH = Sucrose.Player.CS.Helper.VideoHelper;
using SPCSMI = Sucrose.Player.CS.Manage.Internal;
using SPSEH = Sucrose.Player.Shared.Event.Handler;
using SPSHS = Sucrose.Player.Shared.Helper.Source;
using SSEST = Sucrose.Space.Enum.StretchType;

namespace Sucrose.Player.CS.View
{
    /// <summary>
    /// Interaction logic for VideoEngine.xaml
    /// </summary>
    public sealed partial class VideoEngine : Window
    {
        private readonly DispatcherTimer Timer = new();

        public VideoEngine(string Video)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SPSEH.ContentRendered(this);

            SPCSMI.CefPlayer.MenuHandler = new CustomContextMenuHandler();

            Content = SPCSMI.CefPlayer;

            SPCSMI.CefPlayer.Address = SPSHS.GetSource(Video).ToString();
            //SPCSMI.CefPlayer.Address = @"http://www.bokowsky.net/de/knowledge-base/video/videos/big_buck_bunny_240p.ogg"; //.webm - .mp4 - .ogg

            SPCSMI.CefPlayer.BrowserSettings = SPCSMI.CefSettings;

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SPCSMI.CefPlayer.FrameLoadEnd += SPCSEVH.CefPlayerFrameLoadEnd;
            SPCSMI.CefPlayer.Loaded += SPCSEVH.CefPlayerLoaded;

            Closing += (s, e) => SPCSMI.CefPlayer.Dispose();
            Loaded += (s, e) => SPSEH.WindowLoaded(this);
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            SPCSHVH.SetLoop(SMMI.EngineSettingManager.GetSetting(SMC.Loop, true));

            SPCSHVH.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SSEST Stretch = SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSEST.Fill);

            if ((int)Stretch < Enum.GetValues(typeof(SSEST)).Length)
            {
                SPCSHVH.SetStretch(Stretch);
            }
        }
    }
}