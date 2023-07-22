using System.Diagnostics;
using System.IO;
using System.Windows;
using SEAAHA = Sucrose.Engine.AA.Helper.Application;
using SEAAHR = Sucrose.Engine.AA.Helper.Ready;
using SEAAMI = Sucrose.Engine.AA.Manage.Internal;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SWHPI = Skylark.Wing.Helper.ProcessInterop;

namespace Sucrose.Engine.AA.View
{
    /// <summary>
    /// Interaction logic for Application.xaml
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

            Closing += (s, e) => SSSHP.Kill(SEAAMI.Application);
            Closed += (s, e) => SSSHP.Kill(SEAAMI.Application);
            Loaded += (s, e) => SESEH.WindowLoaded(this);

            SSSHP.Run(SEAAMI.Application, SEAAMI.ApplicationArguments, ProcessWindowStyle.Normal);

            do
            {
                if (SSSHP.Work(SEAAMI.Application))
                {
                    SEAAMI.ApplicationProcess = SSSHP.Get(SEAAMI.Application);

                    SEAAMI.ApplicationHandle = SWHPI.MainWindowHandle(SEAAMI.ApplicationProcess);
                }

                Task.Delay(100).Wait();
            } while (SEAAHR.Check());

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            SESEH.ApplicationLoaded(SEAAMI.ApplicationProcess);
            SESEH.ApplicationRendered(SEAAMI.ApplicationProcess);

            SEAAHA.SetVolume(SESHD.GetVolume());
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            SEAAHA.SetVolume(SESHD.GetVolume());

            if (!SSSHP.Work(SEAAMI.Application))
            {
                Close();
            }
        }
    }
}