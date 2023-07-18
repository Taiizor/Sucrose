using System.Diagnostics;
using System.IO;
using System.Windows;
using SEAAHA = Sucrose.Engine.AA.Helper.Application;
using SEAAHR = Sucrose.Engine.AA.Helper.Ready;
using SEAAMI = Sucrose.Engine.AA.Manage.Internal;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
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
        public Application(string Application, string Arguments)
        {
            InitializeComponent();

            SEAAMI.Application = Application;
            SEAAMI.ApplicationArguments = Arguments;
            SEAAMI.ApplicationName = Path.GetFileName(Application);
            SMMI.AuroraSettingManager.SetSetting(SMC.App, SEAAMI.ApplicationName);

            Closing += (s, e) => SSHP.Kill(SEAAMI.Application);
            Closed += (s, e) => SSHP.Kill(SEAAMI.Application);
            Loaded += (s, e) => SESEH.WindowLoaded(this);

            SSHP.Run(SEAAMI.Application, SEAAMI.ApplicationArguments, ProcessWindowStyle.Normal);

            do
            {
                if (SSHP.Work(SEAAMI.Application))
                {
                    SEAAMI.ApplicationProcess = SSHP.Get(SEAAMI.Application);

                    SEAAMI.ApplicationHandle = SWHPI.MainWindowHandle(SEAAMI.ApplicationProcess);
                }

                Task.Delay(100).Wait();
            } while (SEAAHR.Check());

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            SESEH.ApplicationLoaded(SEAAMI.ApplicationProcess);
            SESEH.ApplicationRendered(SEAAMI.ApplicationProcess);

            SEAAHA.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            SEAAHA.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            if (!SSHP.Work(SEAAMI.Application))
            {
                Close();
            }
        }
    }
}