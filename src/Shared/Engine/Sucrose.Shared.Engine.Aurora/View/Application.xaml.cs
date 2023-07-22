using System.Diagnostics;
using System.IO;
using System.Windows;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSEAHA = Sucrose.Shared.Engine.Aurora.Helper.Application;
using SSEAHR = Sucrose.Shared.Engine.Aurora.Helper.Ready;
using SSEAMI = Sucrose.Shared.Engine.Aurora.Manage.Internal;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SWHPI = Skylark.Wing.Helper.ProcessInterop;

namespace Sucrose.Shared.Engine.Aurora.View
{
    /// <summary>
    /// Interaction logic for Application.xaml
    /// </summary>
    public sealed partial class Application : Window
    {
        public Application(string Application, string Arguments)
        {
            InitializeComponent();

            SSEAMI.Application = Application;
            SSEAMI.ApplicationArguments = Arguments;
            SSEAMI.ApplicationName = Path.GetFileName(Application);
            SMMI.AuroraSettingManager.SetSetting(SMC.App, SSEAMI.ApplicationName);

            Closing += (s, e) => SSSHP.Kill(SSEAMI.Application);
            Closed += (s, e) => SSSHP.Kill(SSEAMI.Application);
            Loaded += (s, e) => SESEH.WindowLoaded(this);

            SSSHP.Run(SSEAMI.Application, SSEAMI.ApplicationArguments, ProcessWindowStyle.Normal);

            do
            {
                if (SSSHP.Work(SSEAMI.Application))
                {
                    SSEAMI.ApplicationProcess = SSSHP.Get(SSEAMI.Application);

                    SSEAMI.ApplicationHandle = SWHPI.MainWindowHandle(SSEAMI.ApplicationProcess);
                }

                Task.Delay(100).Wait();
            } while (SSEAHR.Check());

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            SESEH.ApplicationLoaded(SSEAMI.ApplicationProcess);
            SESEH.ApplicationRendered(SSEAMI.ApplicationProcess);

            SSEAHA.SetVolume(SESHD.GetVolume());
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            SSEAHA.SetVolume(SESHD.GetVolume());

            if (!SSSHP.Work(SSEAMI.Application))
            {
                Close();
            }
        }
    }
}