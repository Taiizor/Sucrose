using System.Windows;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SEWVEV = Sucrose.Engine.WV.Event.Video;
using SEWVHV = Sucrose.Engine.WV.Helper.Video;
using SEWVMI = Sucrose.Engine.WV.Manage.Internal;

namespace Sucrose.Engine.WV.View
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

            Content = SEWVMI.WebEngine;

            SEWVMI.Video = Video;

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            SEWVMI.WebEngine.CoreWebView2InitializationCompleted += SEWVEV.WebEngineInitializationCompleted;

            Closing += (s, e) => SEWVMI.WebEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            if (SESMI.Initialized)
            {
                SEWVHV.SetLoop(SESHD.GetLoop());

                SEWVHV.SetVolume(SESHD.GetVolume());

                SEWVHV.SetStretch(SESHD.GetStretch());
            }
        }
    }
}