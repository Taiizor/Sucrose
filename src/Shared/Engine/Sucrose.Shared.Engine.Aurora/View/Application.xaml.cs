using Microsoft.Win32;
using System.Diagnostics;
using System.IO;
using System.Windows;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSEAHA = Sucrose.Shared.Engine.Aurora.Helper.Application;
using SSEAHR = Sucrose.Shared.Engine.Aurora.Helper.Ready;
using SSEAMI = Sucrose.Shared.Engine.Aurora.Manage.Internal;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SWUS = Skylark.Wing.Utility.Screene;

namespace Sucrose.Shared.Engine.Aurora.View
{
    /// <summary>
    /// Interaction logic for Application.xaml
    /// </summary>
    public sealed partial class Application : Window, IDisposable
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
            Loaded += (s, e) => SSEEH.WindowLoaded(this);

            SWUS.Initialize();

            int ScreenCount = SWUS.Screens.Count();

            for (int Count = 0; Count < ScreenCount; Count++)
            {
                SSEMI.Applications.Add(SSSHP.Run(SSEAMI.Application, SSEAMI.ApplicationArguments, ProcessWindowStyle.Normal));
            }

            do
            {
                Task.Delay(250).Wait();
            } while (SSEAHR.Check(ScreenCount));

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            SSEMI.Applications.ForEach(Application =>
            {
                SSEEH.ApplicationLoaded(Application);
                SSEEH.ApplicationRendered(Application);
            });

            SystemEvents.DisplaySettingsChanged += (s, e) => SSEMI.Applications.ForEach(Application => SSEEH.DisplaySettingsChanged(Application, DateTime.Now));

            SSEAHA.SetVolume(SSEHD.GetVolume());
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            Dispose();

            SSEHR.Control();

            SSEAHA.SetVolume(SSEHD.GetVolume());

            if (!SSSHP.Work(SSEAMI.Application))
            {
                Close();
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}