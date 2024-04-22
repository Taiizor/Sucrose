using Microsoft.Win32;
using System.Windows;
using System.Windows.Media;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEVEG = Sucrose.Shared.Engine.Vexana.Event.Gif;
using SSEVHG = Sucrose.Shared.Engine.Vexana.Helper.Gif;
using SSEVHP = Sucrose.Shared.Engine.Vexana.Helper.Parse;
using SSEVMI = Sucrose.Shared.Engine.Vexana.Manage.Internal;

namespace Sucrose.Shared.Engine.Vexana.View
{
    /// <summary>
    /// Interaction logic for Gif.xaml
    /// </summary>
    public sealed partial class Gif : Window, IDisposable
    {
        public Gif(string Gif)
        {
            InitializeComponent();

            SystemEvents.DisplaySettingsChanged += (s, e) => SSEEH.DisplaySettingsChanged(this, DateTime.Now);

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            Content = SSEVMI.ImageEngine;

            SSEVMI.ImageResult = SSEVHP.Gif(SSEHS.GetSource(Gif).ToString());

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            Closing += (s, e) => SSEVMI.ImageTimer.Stop();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);

            SSEVMI.ImageTimer.Tick += new EventHandler(SSEVEG.ImageTimer_Tick);
            SSEVMI.ImageTimer.Start();
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            Dispose();

            SSEHR.Control();

            SSEVHG.SetLoop(SSEHD.GetLoop());

            SSEVMI.ImageEngine.Stretch = (Stretch)SSEHD.GetStretch();

            if (SMMM.PausePerformance)
            {
                SSEVHG.Pause();

                SSEMI.PausePerformance = true;
            }
            else if (SSEMI.PausePerformance)
            {
                SSEVHG.Play();

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