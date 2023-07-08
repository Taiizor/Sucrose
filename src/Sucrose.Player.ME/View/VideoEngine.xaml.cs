using System.Windows;
using System.Windows.Media;
using System.Windows.Threading;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMEEVH = Sucrose.Player.ME.Event.VideoHandler;
using SPMEHVH = Sucrose.Player.ME.Helper.VideoHelper;
using SPMEMI = Sucrose.Player.ME.Manage.Internal;
using SPSEH = Sucrose.Player.Shared.Event.Handler;
using SPSHS = Sucrose.Player.Shared.Helper.Source;
using SSEST = Sucrose.Space.Enum.StretchType;

namespace Sucrose.Player.ME.View
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

            Content = SPMEMI.MediaPlayer;

            SPMEMI.MediaPlayer.Source = SPSHS.GetSource(Video);

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SPMEMI.MediaPlayer.MediaEnded += SPMEEVH.MediaPlayerEnded;

            Closing += (s, e) => SPMEMI.MediaPlayer.Close();
            Loaded += (s, e) => SPSEH.WindowLoaded(this);

            SPMEHVH.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SPMEHVH.Play();
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            SPMEHVH.SetLoop(SMMI.EngineSettingManager.GetSetting(SMC.Loop, true));

            SPMEHVH.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SSEST Stretch = SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSEST.Fill);

            if ((int)Stretch < Enum.GetValues(typeof(SSEST)).Length)
            {
                SPMEMI.MediaPlayer.Stretch = (Stretch)Stretch;
            }
        }
    }
}