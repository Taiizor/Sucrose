using System.Diagnostics;
using System.IO;
using System.Windows;
using System.Windows.Threading;
using SEAAHA = Sucrose.Engine.AA.Helper.Application;
using SEAAMI = Sucrose.Engine.AA.Manage.Internal;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSHP = Sucrose.Space.Helper.Processor;
using SWHPI = Skylark.Wing.Helper.ProcessInterop;

namespace Sucrose.Engine.AA.View
{
    /// <summary>
    /// Interaction logic for Video.xaml
    /// </summary>
    public sealed partial class Application : Window
    {
        private readonly DispatcherTimer Timer = new();

        public Application(string Application, string Arguments)
        {
            InitializeComponent();

            SEAAMI.Application = Application;
            SEAAMI.ApplicationName = Path.GetFileName(Application);
            SMMI.AuroraSettingManager.SetSetting(SMC.App, SEAAMI.ApplicationName);

            SSHP.Run(SEAAMI.Application, Arguments, ProcessWindowStyle.Minimized);

            Closing += (s, e) => SSHP.Kill(SEAAMI.Application);
            Closed += (s, e) => SSHP.Kill(SEAAMI.Application);
            Loaded += (s, e) => SESEH.WindowLoaded(this);

            do
            {
                SEAAMI.ApplicationProcess = SSHP.Get(SEAAMI.Application);

                Task.Delay(500).Wait();
            } while (SEAAMI.ApplicationProcess == null || !SSHP.Work(SEAAMI.Application));

            Task.Delay(1000).Wait();

            SEAAMI.ApplicationHandle = SWHPI.MainWindowHandle(SEAAMI.ApplicationProcess);

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SESEH.ApplicationLoaded(SEAAMI.ApplicationProcess);
            SESEH.ApplicationRendered(SEAAMI.ApplicationProcess);

            SEAAHA.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            SEAAHA.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            if (!SSHP.Work(SEAAMI.Application))
            {
                Close();
            }
        }
    }
}