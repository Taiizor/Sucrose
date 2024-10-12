using Microsoft.Win32;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Animation;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEXHG = Sucrose.Shared.Engine.Xavier.Helper.Gif;
using SSEXMI = Sucrose.Shared.Engine.Xavier.Manage.Internal;
using SXAGAB = Sucrose.XamlAnimatedGif.AnimationBehavior;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMMCB = Sucrose.Memory.Manage.Constant.Backgroundog;

namespace Sucrose.Shared.Engine.Xavier.View
{
    /// <summary>
    /// Interaction logic for Gif.xaml
    /// </summary>
    public sealed partial class Gif : Window, IDisposable
    {
        public Gif(string Gif)
        {
            InitializeComponent();

            SystemEvents.DisplaySettingsChanged += (s, e) => SSEEH.DisplaySettingsChanged(this);

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            Content = SSEXMI.ImageEngine;

            SXAGAB.SetRepeatBehavior(SSEXMI.ImageEngine, RepeatBehavior.Forever);
            SXAGAB.SetSourceUri(SSEXMI.ImageEngine, SSEHS.GetSource(Gif));
            SXAGAB.SetCacheFramesInMemory(SSEXMI.ImageEngine, false);
            SXAGAB.SetAnimateInDesignMode(SSEXMI.ImageEngine, false);
            SXAGAB.SetAutoStart(SSEXMI.ImageEngine, true);

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            Closing += (s, e) => SSEMI.GeneralTimer.Stop();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            Dispose();

            SSEHR.Control();

            SSEXHG.SetMemory(false);

            SSEXHG.SetLoop(SSEHD.GetLoop());

            SSEXMI.ImageEngine.Stretch = (Stretch)SSEHD.GetStretch();

            if (SMMB.PausePerformance)
            {
                SSEXHG.Pause();

                SSEMI.PausePerformance = true;
            }
            else if (SSEMI.PausePerformance)
            {
                SSEXHG.Resume();

                SSEMI.PausePerformance = false;
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}