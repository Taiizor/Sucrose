using System.Windows;
using System.Windows.Media;
using System.Windows.Threading;
using SEMEEV = Sucrose.Engine.ME.Event.Video;
using SEMEHV = Sucrose.Engine.ME.Helper.Video;
using SEMEMI = Sucrose.Engine.ME.Manage.Internal;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSEST = Sucrose.Space.Enum.StretchType;

namespace Sucrose.Engine.ME.View
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

            Content = SEMEMI.MediaEngine;

            SEMEMI.MediaEngine.Source = SESHS.GetSource(Video);

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SEMEMI.MediaEngine.MediaEnded += SEMEEV.MediaEngineEnded;

            Closing += (s, e) => SEMEMI.MediaEngine.Close();
            Loaded += (s, e) => SESEH.WindowLoaded(this);

            SEMEHV.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SEMEHV.Play();
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            SEMEHV.SetLoop(SMMI.EngineSettingManager.GetSetting(SMC.Loop, true));

            SEMEHV.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SSEST Stretch = SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSEST.Fill);

            if ((int)Stretch < Enum.GetValues(typeof(SSEST)).Length)
            {
                SEMEMI.MediaEngine.Stretch = (Stretch)Stretch;
            }
        }
    }
}