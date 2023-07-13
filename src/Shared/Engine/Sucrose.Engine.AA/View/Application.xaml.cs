using System.Diagnostics;
using System.IO;
using System.Windows;
using System.Windows.Media;
using System.Windows.Threading;
using SDEST = Sucrose.Dependency.Enum.StretchType;
using SEAAEV = Sucrose.Engine.AA.Event.Video;
using SEAAHA = Sucrose.Engine.AA.Helper.Application;
using SEAAMI = Sucrose.Engine.AA.Manage.Internal;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSHP = Sucrose.Space.Helper.Processor;

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

            SSHP.Run(SEAAMI.Application, Arguments, ProcessWindowStyle.Maximized);

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            Closing += (s, e) => SSHP.Kill(SEAAMI.Application);
            Closed += (s, e) => SSHP.Kill(SEAAMI.Application);
            Loaded += (s, e) => SESEH.WindowLoaded(this);

            SEAAHA.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            while(SEAAMI.ApplicationProcess == null)
            {
                SEAAMI.ApplicationProcess = SSHP.Get(SEAAMI.Application);
            }

            SESEH.ApplicationLoaded(SEAAMI.ApplicationProcess);
            SESEH.ApplicationRendered(SEAAMI.ApplicationProcess);
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            SEAAHA.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SDEST Stretch = SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SDEST.Fill);

            if ((int)Stretch < Enum.GetValues(typeof(SDEST)).Length)
            {
                //SEAAMI.MediaEngine.Stretch = (Stretch)Stretch; //Açılan uygulama boyutu değiştirilecek ya da hiç olmayacak
            }

            if (!SSHP.Work(SEAAMI.Application))
            {
                Close();
            }

            SESEH.ApplicationLoaded(SEAAMI.ApplicationProcess);
        }
    }
}