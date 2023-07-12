using Microsoft.Web.WebView2.Core;
using System.Windows;
using System.Windows.Threading;
using SDEST = Sucrose.Dependency.Enum.StretchType;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
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
        private readonly DispatcherTimer Timer = new();

        public Video(string Video)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SESEH.ContentRendered(this);

            Content = SEWVMI.WebEngine;

            SEWVMI.Video = Video;

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SEWVMI.WebEngine.CoreWebView2InitializationCompleted += SEWVEV.WebEngineInitializationCompleted;

            Closing += (s, e) => SEWVMI.WebEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }

        private void CoreWebView2_DOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            throw new NotImplementedException();
        }

        private void Timer_Tick(object sender, EventArgs e)
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