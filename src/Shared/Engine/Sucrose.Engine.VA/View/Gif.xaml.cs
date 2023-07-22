using System.Windows;
using System.Windows.Media;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SEVAEG = Sucrose.Engine.VA.Event.Gif;
using SEVAHG = Sucrose.Engine.VA.Helper.Gif;
using SEVAHP = Sucrose.Engine.VA.Helper.Parse;
using SEVAMI = Sucrose.Engine.VA.Manage.Internal;

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
            SEVAHG.SetLoop(SESHD.GetLoop());

            SEVAMI.ImageEngine.Stretch = (Stretch)SESHD.GetStretch();
        }
    }
}