using System.Windows;
using System.Windows.Threading;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SEWVEYT = Sucrose.Engine.WV.Event.YouTube;
using SEWVHYT = Sucrose.Engine.WV.Helper.YouTube;
using SEWVMI = Sucrose.Engine.WV.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Engine.WV.View
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

            Content = SEWVMI.WebEngine;

            SEWVMI.YouTube = YouTube;

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SEWVMI.WebEngine.CoreWebView2InitializationCompleted += SEWVEYT.WebEngineInitializationCompleted;

            Closing += (s, e) => SEWVMI.WebEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            SEWVHYT.SetLoop(SMMI.EngineSettingManager.GetSetting(SMC.Loop, true));

            SEWVHYT.SetShuffle(SMMI.EngineSettingManager.GetSetting(SMC.Shuffle, true));

            SEWVHYT.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));
        }
    }
}