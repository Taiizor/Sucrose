using System.Windows;
using System.Windows.Media;
using SENAEV = Sucrose.Engine.NA.Event.Video;
using SENAHV = Sucrose.Engine.NA.Helper.Video;
using SENAMI = Sucrose.Engine.NA.Manage.Internal;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;

namespace Sucrose.Engine.NA.View
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

            Content = SENAMI.MediaEngine;

            SENAMI.MediaEngine.Source = SESHS.GetSource(Video);

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            SENAMI.MediaEngine.MediaEnded += SENAEV.MediaEngineEnded;

            Closing += (s, e) => SENAMI.MediaEngine.Close();
            Loaded += (s, e) => SESEH.WindowLoaded(this);

            SENAHV.SetVolume(SESHD.GetVolume());

            SENAHV.Play();
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            SENAHV.SetLoop(SESHD.GetLoop());

            SENAHV.SetVolume(SESHD.GetVolume());

            SENAMI.MediaEngine.Stretch = (Stretch)SESHD.GetStretch();
        }
    }
}