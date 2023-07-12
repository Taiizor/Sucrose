using System.Windows;
using System.Windows.Threading;
using SECSEV = Sucrose.Engine.CS.Event.Video;
using SECSHV = Sucrose.Engine.CS.Helper.Video;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SDEST = Sucrose.Dependency.Enum.StretchType;

namespace Sucrose.Engine.CS.View
{
    /// <summary>
    /// Interaction logic for Video.xaml
    /// </summary>
    public sealed partial class Video : Window
    {
        private readonly DispatcherTimer Timer = new();

        public Video(string Video)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SESEH.ContentRendered(this);

            SECSMI.CefEngine.MenuHandler = new CustomContextMenuHandler();

            Content = SECSMI.CefEngine;

            SECSMI.CefEngine.Address = SESHS.GetSource(Video).ToString();
            //SECSMI.CefEngine.Address = @"http://www.bokowsky.net/de/knowledge-base/video/videos/big_buck_bunny_240p.ogg"; //.webm - .mp4 - .ogg

            SECSMI.CefEngine.BrowserSettings = SECSMI.CefSettings;

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SECSMI.CefEngine.FrameLoadEnd += SECSEV.CefEngineFrameLoadEnd;
            SECSMI.CefEngine.Loaded += SECSEV.CefEngineLoaded;

            Closing += (s, e) => SECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }

        private void Timer_Tick(object sender, EventArgs e)
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