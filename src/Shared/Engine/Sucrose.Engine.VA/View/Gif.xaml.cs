using System.Windows;
using System.Windows.Media;
using SDEST = Sucrose.Dependency.Enum.StretchType;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
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
        public Gif(string Gif)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SESEH.ContentRendered(this);

            Content = SEVAMI.ImageEngine;

            SEVAMI.ImageResult = SEVAHP.Gif(SESHS.GetSource(Gif).ToString());

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            Closing += (s, e) => SEVAMI.ImageTimer.Stop();
            Loaded += (s, e) => SESEH.WindowLoaded(this);

            SEVAMI.ImageTimer.Tick += new EventHandler(SEVAEG.ImageTimer_Tick);
            SEVAMI.ImageTimer.Start();
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
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