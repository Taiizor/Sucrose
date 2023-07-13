using System.Windows;
using System.Windows.Media;
using System.Windows.Threading;
using SDEST = Sucrose.Dependency.Enum.StretchType;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SEVAEG = Sucrose.Engine.VA.Event.Gif;
using SEVAHG = Sucrose.Engine.VA.Helper.Gif;
using SEVAHP = Sucrose.Engine.VA.Helper.Parse;
using SEVAMI = Sucrose.Engine.VA.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Engine.VA.View
{
    /// <summary>
    /// Interaction logic for Gif.xaml
    /// </summary>
    public sealed partial class Gif : Window
    {
        private readonly DispatcherTimer Timer = new();

        public Gif(string Gif)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SESEH.ContentRendered(this);

            Content = SEVAMI.ImageEngine;

            SEVAMI.ImageResult = SEVAHP.Gif(SESHS.GetSource(Gif).ToString());

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            Closing += (s, e) => SEVAMI.ImageTimer.Stop();
            Loaded += (s, e) => SESEH.WindowLoaded(this);

            SEVAMI.ImageTimer.Tick += new EventHandler(SEVAEG.ImageTimer_Tick);
            SEVAMI.ImageTimer.Start();
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            SEVAHG.SetLoop(SMMI.EngineSettingManager.GetSetting(SMC.Loop, true));

            SDEST Stretch = SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SDEST.Fill);

            if ((int)Stretch < Enum.GetValues(typeof(SDEST)).Length)
            {
                SEVAMI.ImageEngine.Stretch = (Stretch)Stretch;
            }
        }
    }
}