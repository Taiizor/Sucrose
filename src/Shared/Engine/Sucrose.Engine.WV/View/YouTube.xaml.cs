using System.Windows;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SEWVEYT = Sucrose.Engine.WV.Event.YouTube;
using SEWVHYT = Sucrose.Engine.WV.Helper.YouTube;
using SEWVMI = Sucrose.Engine.WV.Manage.Internal;

namespace Sucrose.Engine.WV.View
{
    /// <summary>
    /// Interaction logic for YouTube.xaml
    /// </summary>
    public sealed partial class YouTube : Window
    {
        public YouTube(string YouTube)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SESEH.ContentRendered(this);

            Content = SEWVMI.WebEngine;

            SEWVMI.YouTube = YouTube;

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            SEWVMI.WebEngine.CoreWebView2InitializationCompleted += SEWVEYT.WebEngineInitializationCompleted;

            Closing += (s, e) => SEWVMI.WebEngine.Dispose();
            Loaded += (s, e) => SESEH.WindowLoaded(this);
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            if (SEWVMI.Initialized)
            {
                SEWVHYT.First();

                SEWVHYT.SetLoop(SESHD.GetLoop());

                SEWVHYT.SetVolume(SESHD.GetVolume());

                SEWVHYT.SetShuffle(SESHD.GetShuffle());
            }
        }
    }
}