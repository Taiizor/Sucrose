using System.Windows;
using System.Windows.Media;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSENEV = Sucrose.Shared.Engine.Nebula.Event.Video;
using SSENHV = Sucrose.Shared.Engine.Nebula.Helper.Video;
using SSENMI = Sucrose.Shared.Engine.Nebula.Manage.Internal;

namespace Sucrose.Shared.Engine.Nebula.View
{
    /// <summary>
    /// Interaction logic for Video.xaml
    /// </summary>
    public sealed partial class Video : Window, IDisposable
    {
        public Video(string Video)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            Content = SSENMI.MediaEngine;

            SSENMI.MediaEngine.Source = SSEHS.GetSource(Video);

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            SSENMI.MediaEngine.MediaEnded += SSENEV.MediaEngineEnded;

            Closing += (s, e) => SSENMI.MediaEngine.Close();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);

            SSENHV.SetVolume(SSEHD.GetVolume());

            SSENHV.Play();
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            Dispose();

            SSENHV.SetLoop(SSEHD.GetLoop());

            SSENHV.SetVolume(SSEHD.GetVolume());

            SSENMI.MediaEngine.Stretch = (Stretch)SSEHD.GetStretch();
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}