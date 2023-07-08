using Microsoft.Web.WebView2.Core;
using System.Windows;
using System.Windows.Threading;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPSEH = Sucrose.Player.Shared.Event.Handler;
using SPWVEVH = Sucrose.Player.WV.Event.VideoHandler;
using SPWVHVH = Sucrose.Player.WV.Helper.VideoHelper;
using SPWVMI = Sucrose.Player.WV.Manage.Internal;
using SSEST = Sucrose.Space.Enum.StretchType;

namespace Sucrose.Player.WV.View
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

            Content = SPWVMI.EdgePlayer;

            SPWVMI.Video = Video;

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SPWVMI.EdgePlayer.CoreWebView2InitializationCompleted += SPWVEVH.EdgePlayerInitializationCompleted;

            Closing += (s, e) => SPWVMI.EdgePlayer.Dispose();
            Loaded += (s, e) => SPSEH.WindowLoaded(this);
        }

        private void CoreWebView2_DOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            throw new NotImplementedException();
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            SPWVHVH.SetLoop(SMMI.EngineSettingManager.GetSetting(SMC.Loop, true));

            SPWVHVH.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SSEST Stretch = SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSEST.Fill);

            if ((int)Stretch < Enum.GetValues(typeof(SSEST)).Length)
            {
                SPWVHVH.SetStretch(Stretch);
            }
        }
    }
}