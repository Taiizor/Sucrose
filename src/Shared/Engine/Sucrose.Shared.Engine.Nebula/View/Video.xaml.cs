using System.Windows;
using System.Windows.Media;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SSENEV = Sucrose.Shared.Engine.Nebula.Event.Video;
using SSENHV = Sucrose.Shared.Engine.Nebula.Helper.Video;
using SSENMI = Sucrose.Shared.Engine.Nebula.Manage.Internal;

namespace Sucrose.Shared.Engine.Nebula.View
{
    /// <summary>
    /// Interaction logic for Video.xaml
    /// </summary>
    public sealed partial class Video : Window
    {
        public Video(string Video)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SESEH.ContentRendered(this);

            Content = SSENMI.MediaEngine;

            SSENMI.MediaEngine.Source = SESHS.GetSource(Video);

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            SSENMI.MediaEngine.MediaEnded += SSENEV.MediaEngineEnded;

            Closing += (s, e) => SSENMI.MediaEngine.Close();
            Loaded += (s, e) => SESEH.WindowLoaded(this);

            SSENHV.SetVolume(SESHD.GetVolume());

            SSENHV.Play();
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            SSENHV.SetLoop(SESHD.GetLoop());

            SSENHV.SetVolume(SESHD.GetVolume());

            SSENMI.MediaEngine.Stretch = (Stretch)SESHD.GetStretch();
        }
    }
}