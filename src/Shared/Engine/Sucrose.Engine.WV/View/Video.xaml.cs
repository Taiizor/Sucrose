using System.Windows;
using SDEST = Sucrose.Dependency.Enum.StretchType;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SEWVEV = Sucrose.Engine.WV.Event.Video;
using SEWVHV = Sucrose.Engine.WV.Helper.Video;
using SEWVMI = Sucrose.Engine.WV.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Engine.WV.View
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

            Content = SEWVMI.WebEngine;

            SEWVMI.Video = Video;

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            SEWVMI.WebEngine.CoreWebView2InitializationCompleted += SEWVEV.WebEngineInitializationCompleted;

            Closing += (s, e) => SEWVMI.WebEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            SEWVHV.SetLoop(SMMI.EngineSettingManager.GetSetting(SMC.Loop, true));

            SEWVHV.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SDEST Stretch = SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SDEST.Fill);

            if ((int)Stretch < Enum.GetValues(typeof(SDEST)).Length)
            {
                SEWVHV.SetStretch(Stretch);
            }
        }
    }
}